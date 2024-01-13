// Copyright 2024 Jeremy Edwards
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    common::{
        checksum::FileChecksum,
        db::FileChecksumDB,
        fs::{threaded_walk_dir, FileMetadata},
    },
    duplicate::db::DuplicateFileDB,
};
use crossbeam_channel::Receiver;
use indicatif::ProgressBar;
use log::{trace, warn};
use std::{
    io::{self, Write},
    path::Path,
};
mod db;
mod report;

#[derive(clap::Args, Clone)]
pub(crate) struct Args {
    /// List of paths to scan.
    #[arg(long, default_value = ".")]
    pub(crate) path: Vec<std::path::PathBuf>,
    /// Minimum size of file to consider while scanning.
    #[arg(long, default_value_t = 0)]
    pub(crate) min_size: u64,
    /// List of patterns that match files that should be deleted if they are in a group of duplicates.
    #[arg(long, default_value = "")]
    pub(crate) delete_pattern: Vec<String>,
    /// If false, will perform the delete based on the pattern filtering provided by --delete_pattern.
    #[arg(long, default_value_t = true)]
    pub(crate) dry_run: std::primitive::bool,
    /// Path of where the duplicate file report will be written.
    #[arg(long, default_value = "duplicates.html")]
    pub(crate) output: String,
    /// If true, if an existing report already exists it will be overwritten.
    #[arg(long, default_value_t = true)]
    pub(crate) overwrite: std::primitive::bool,
    /// Path of where the checksum db is stored. This file makes subsequent runs much faster.
    #[arg(long, default_value = "checksums.txt")]
    pub(crate) db: std::path::PathBuf,
    /// Path where the file list of all duplicates that should be deleted are recorded.
    #[arg(long, default_value = "rmlist.txt")]
    pub(crate) rmlist: std::path::PathBuf,
    /// Number of threads for calculating checksums.
    #[arg(long, default_value_t = 2)]
    pub(crate) checksum_threads: usize,
}

fn create_progress_bars() -> (ProgressBar, ProgressBar, ProgressBar, ProgressBar) {
    let progress_factory = crate::common::progress::ProgressFactory::new();
    let pb_title = progress_factory.create_title();
    pb_title.set_prefix("Duplicate");
    (
        pb_title,
        progress_factory.create_detail(),
        progress_factory.create_bar(),
        progress_factory.create_danger(),
    )
}

fn scan_files(
    pb_detail: &ProgressBar,
    path_rx: Receiver<FileMetadata>,
    dup_db: &mut DuplicateFileDB,
    min_size: u64,
) -> u64 {
    let mut files_scanned = 0;
    for md in path_rx {
        files_scanned += 1;
        if md.size >= min_size {
            pb_detail.set_message(format!("[{files_scanned}] {:#?}", md.path));
            dup_db.put(&md);
        }
    }
    files_scanned
}

pub(crate) fn run(args: &Args) -> io::Result<()> {
    let (path_tx, path_rx) = crossbeam_channel::unbounded();
    let (hash_tx, hash_rx) = crossbeam_channel::unbounded();
    let (hash_result_tx, hash_result_rx) = crossbeam_channel::unbounded::<FileChecksum>();
    let (pb_title, pb_detail, pb_checksum_bar, pb_delete_bar) = create_progress_bars();

    pb_title.set_message("Scanning Files...");
    let delete_pattern = args.delete_pattern.clone();

    let params = args.clone();
    let report_title = get_report_title(&args.path);
    let walk_join = threaded_walk_dir(&args.path, path_tx)?;
    let duplicate_thread = std::thread::spawn(move || {
        let mut dup_db = DuplicateFileDB::new();
        let mut checksum_db = FileChecksumDB::new();
        match checksum_db.load(&params.db) {
            Ok(()) => {}
            Err(error) => warn!(
                "cannot load checksum file {checksum_db_filepath:#?}, {error}",
                checksum_db_filepath = params.db
            ),
        }

        pb_detail.set_prefix("Scan");
        let files_scanned = scan_files(&pb_detail, path_rx, &mut dup_db, params.min_size);

        dup_db.remove_unique_size();

        let num_candidates = dup_db.m.len();
        pb_title.set_message(format!(
            "Possible Duplicates: {num_candidates}/{files_scanned} files, calculating checksums..."
        ));
        pb_detail.set_message("Checksumming files...");

        let mut require_checksum = 0;
        let mut require_checksum_size = 0;
        for md in dup_db.m.values() {
            if checksum_db.get(md).is_none() {
                hash_tx.send(md.path.clone()).unwrap();
                require_checksum += 1;
                require_checksum_size += md.size;
            }
        }
        drop(hash_tx);

        pb_checksum_bar.set_length(require_checksum);
        let require_checksum_size_str = crate::common::util::human_size(require_checksum_size);
        pb_title.set_message(format!("{num_candidates} of {files_scanned} files are possible duplicates, calculating {require_checksum} checksums ({require_checksum_size_str})..."));

        checksum_candidate_files(
            &params,
            &pb_detail,
            &pb_checksum_bar,
            &mut dup_db,
            &mut checksum_db,
            num_candidates,
            require_checksum,
            &hash_result_rx,
        );

        pb_detail.set_message("Calculating duplicates...");
        let delete_files =
            get_duplicate_files_to_delete(&mut dup_db, &checksum_db, &delete_pattern);

        let num_delete: u64 = delete_files
            .len()
            .try_into()
            .expect("cannot convert len to u64.");

        create_rmlist(&delete_files, &params.rmlist);

        let delete_size = delete_matched_duplicate_files(
            &pb_detail,
            &pb_delete_bar,
            &delete_files,
            num_delete,
            params.dry_run,
        );

        pb_detail.set_prefix("Report");
        pb_detail.set_message("Writing...");
        let (dups, num_dups, dup_size) = filter_duplicates(&mut dup_db, &checksum_db);

        let closing_status = get_closing_status(
            files_scanned,
            num_dups,
            dup_size,
            num_delete,
            delete_size,
            params.dry_run,
        );
        write_report(
            &pb_title,
            &pb_detail,
            &params.output,
            &report_title,
            &dups,
            &closing_status,
            params.overwrite,
        );
    });

    let hash_worker_joiner =
        crate::common::checksum::worker_pool(args.checksum_threads, hash_result_tx, &hash_rx);

    walk_join();
    hash_worker_joiner();
    duplicate_thread.join().unwrap();
    Ok(())
}

fn checksum_candidate_files(
    params: &Args,
    pb_detail: &ProgressBar,
    pb_checksum_bar: &ProgressBar,
    dup_db: &mut DuplicateFileDB,
    checksum_db: &mut FileChecksumDB,
    num_candidates: usize,
    require_checksum: u64,
    hash_result_rx: &Receiver<FileChecksum>,
) {
    let hash_batch_size = get_batch_size(num_candidates);
    pb_detail.set_prefix("Checksum");
    let mut num_hash = 0;
    for hash_result in hash_result_rx {
        pb_checksum_bar.inc(1);
        pb_detail.set_message(format!("{:#?}", hash_result.path));
        let p: &std::path::Path = &hash_result.path;
        let checksum: &str = &hash_result.checksum;
        if let Some(dup_val) = dup_db.get(p) {
            checksum_db.put(dup_val, checksum);
            num_hash += 1;
            if num_hash % hash_batch_size == 0 {
                match checksum_db.write(&params.db) {
                    Ok(()) => pb_checksum_bar
                        .set_message(format!("{num_hash}/{require_checksum} checksums")),
                    Err(error) => warn!(
                        "cannot save checksums to db '{checksum_db_filepath:#?}', error:{error}",
                        checksum_db_filepath = params.db
                    ),
                }
            }
        } else {
            warn!("'{p:#?}' has no entry in dup_db.");
        }
    }
    match checksum_db.write(&params.db) {
        Ok(()) => {}
        Err(error) => {
            warn!(
                "cannot save checksums to db '{checksum_db_filepath:#?}', error:{error}",
                checksum_db_filepath = params.db
            );
        }
    }
    pb_checksum_bar.finish_with_message("Done");
    pb_checksum_bar.finish_and_clear();
}

fn get_duplicate_files_to_delete(
    dup_db: &mut DuplicateFileDB,
    checksum_db: &FileChecksumDB,
    delete_pattern: &Vec<String>,
) -> Vec<FileMetadata> {
    let pre_dups = db::get_duplicates(dup_db, checksum_db);
    let mut delete_files = Vec::new();
    for dup in pre_dups.clone() {
        let max_delete_per_group_allowed = dup.len() - 1;
        let mut delete_in_group = 0;
        for md in dup {
            if delete_in_group < max_delete_per_group_allowed
                && match_file(&md.path, delete_pattern)
            {
                delete_files.push(md.clone());
                delete_in_group += 1;
                dup_db.remove(&md);
            }
        }
    }
    delete_files
}

fn create_rmlist(delete_files: &Vec<FileMetadata>, rmlist_path: &Path) {
    if !delete_files.is_empty() {
        match std::fs::File::create(rmlist_path) {
            Ok(file) => {
                let mut writer = std::io::LineWriter::new(file);
                for delete_file in delete_files {
                    let delete_file_path = &delete_file.path;
                    warn!("rmlist {delete_file_path:#?}");
                    if let Some(path_str) = delete_file.path.to_str() {
                        match writer.write_all(path_str.as_bytes()) {
                            Ok(()) => {}
                            Err(error) => {
                                warn!("cannot write line to rmlist '{rmlist_path:#?}' file, error: {error}");
                            }
                        }
                        match writer.write_all("\n".as_bytes()) {
                            Ok(()) => {}
                            Err(error) => {
                                warn!("cannot write line to rmlist '{rmlist_path:#?}' file, error: {error}");
                            }
                        }
                    }
                }
            }
            Err(error) => {
                warn!("cannot write rmlist '{rmlist_path:#?}' file, error: {error}");
            }
        }
    }
}

fn delete_matched_duplicate_files(
    pb_detail: &ProgressBar,
    pb_delete_bar: &ProgressBar,
    delete_files: &Vec<FileMetadata>,
    num_delete: u64,
    dry_run: bool,
) -> u64 {
    pb_detail.set_prefix("Deleting");
    pb_detail.set_message("Removing duplicates...");
    pb_delete_bar.set_length(num_delete);
    let mut delete_size = 0u64;
    for delete_file in delete_files {
        let file_path = delete_file.path.to_str().expect("cannot get file name");
        let size = crate::common::util::human_size(delete_file.size);
        pb_detail.set_message(format!("{file_path} ({size})"));
        pb_delete_bar.inc(1);
        match crate::common::fs::delete_file(file_path, dry_run) {
            Ok(()) => {}
            Err(error) => warn!("failed to delete file {file_path}, error: {error}"),
        }
        delete_size += delete_file.size;
        pb_delete_bar.set_message(crate::common::util::human_size(delete_size));
    }

    pb_delete_bar.finish_and_clear();
    delete_size
}

fn filter_duplicates(
    dup_db: &mut DuplicateFileDB,
    checksum_db: &FileChecksumDB,
) -> (Vec<Vec<FileMetadata>>, u64, u64) {
    dup_db.remove_unique_size();
    let dups = db::get_duplicates(dup_db, checksum_db);
    let mut num_dups = 0u64;
    let mut dup_size = 0u64;
    for dup in dups.clone() {
        for d in dup {
            if let Some(path) = d.path.to_str() {
                let size = d.size;
                let size_str = crate::common::util::human_size(size);
                trace!("Duplicate: {path} -- {size_str}");
                num_dups += 1;
                dup_size += size;
            }
        }
        trace!("------------------------------------");
    }
    (dups, num_dups, dup_size)
}

fn get_closing_status(
    files_scanned: u64,
    num_dups: u64,
    dup_size: u64,
    num_delete: u64,
    delete_size: u64,
    dry_run: bool,
) -> String {
    let delete_size_str = crate::common::util::human_size(delete_size);
    let deleted_text = if dry_run {
        "deleted (dry run)"
    } else {
        "deleted"
    };
    let dup_size_str = crate::common::util::human_size(dup_size);
    format!(
        "Found {num_dups} ({dup_size_str}) duplicates from {files_scanned} files, {num_delete} ({delete_size_str}) were {deleted_text}."
    )
}

fn write_report(
    pb_title: &ProgressBar,
    pb_detail: &ProgressBar,
    output_file: &str,
    report_title: &str,
    dups: &Vec<Vec<FileMetadata>>,
    closing_status: &str,
    overwrite: bool,
) {
    let orig = std::path::Path::new(output_file);
    pb_title.set_message(closing_status.to_string());
    pb_detail.set_message("Writing report...");
    if orig.is_file() && !overwrite {
        pb_detail.set_message(format!("Cannot save report to {output_file} because it already exists, to forcefully overwrite the file use, --overwrite=true"));
    } else {
        match report::html_file(output_file, report_title, dups) {
            Ok(()) => {
                pb_title.finish_with_message(format!("{closing_status} See {output_file}"));
                pb_detail.finish_and_clear();
            }
            Err(error) => {
                pb_detail.set_message(format!(
                    "cannot write report to '{output_file}', error: {error}"
                ));
            }
        }
    }
}

fn get_report_title(path_list: &Vec<std::path::PathBuf>) -> String {
    if path_list.is_empty() {
        String::new()
    } else {
        let path_str_list: Vec<String> = path_list
            .iter()
            .filter_map(|p| p.to_str())
            .map(String::from)
            .collect();
        path_str_list.join(",")
    }
}

fn match_file(path: &std::path::Path, delete_pattern: &Vec<String>) -> bool {
    if let Some(p) = path.to_str() {
        for pattern in delete_pattern {
            if !pattern.is_empty() && p.contains(pattern) {
                return true;
            }
        }
    }
    false
}

fn get_batch_size(num_candidates: usize) -> usize {
    if num_candidates == 0 {
        100
    } else {
        let max_batch_size = num_candidates / 500;
        if max_batch_size > 100 {
            max_batch_size
        } else {
            100
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_batch_size() {
        assert_eq!(get_batch_size(0), 100);
        assert_eq!(get_batch_size(100), 100);
        assert_eq!(get_batch_size(200), 100);
        assert_eq!(get_batch_size(125000), 250);
        assert_eq!(get_batch_size(1500000), 3000);
        assert_eq!(get_batch_size(1250), 100);
    }

    #[test]
    fn run_test() {
        let args = Args {
            path: vec![std::path::PathBuf::from(".")],
            min_size: 0,
            delete_pattern: vec![],
            dry_run: true,
            output: String::new(),
            db: std::path::PathBuf::from("checksums.txt"),
            overwrite: false,
            rmlist: std::path::PathBuf::from("rmlist.txt"),
            checksum_threads: 2,
        };
        run(&args).unwrap();
    }
}

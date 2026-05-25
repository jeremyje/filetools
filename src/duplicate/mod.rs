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

use crate::common::fs::FileMetadata;
use log::{trace, warn};
use std::io;
mod db;
mod pipeline;
mod report;
use clap_verbosity_flag::Verbosity;
use std::time::Duration;

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
    /// If false, will perform the delete based on the pattern filtering provided by `--delete_pattern`.
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
    /// Interval between checksum checkpoints.
    #[arg(long, default_value = "30s", value_parser = humantime::parse_duration)]
    pub(crate) checksum_checkpoint_interval: Duration,
    /// Force deletion of files when the read-only bit is set.
    #[arg(long, default_value_t = false)]
    pub(crate) force: bool,
}

pub(crate) fn run(args: &Args, verbose: Verbosity) -> io::Result<()> {
    let (path_tx, path_rx) = crossbeam_channel::unbounded();
    let (hash_tx, hash_rx) = crossbeam_channel::unbounded();
    let (hash_result_tx, hash_result_rx) =
        crossbeam_channel::unbounded::<crate::common::checksum::FileChecksum>();

    let progress_factory = crate::common::progress::ProgressFactory::new(verbose);
    let pb_title = progress_factory.create_title();
    let pb_detail = progress_factory.create_detail();
    let pb_checksum_bar = progress_factory.create_bar();
    let pb_delete_bar = progress_factory.create_danger();

    pb_title.set_prefix("Duplicate");
    pb_title.set_message("Scanning Files...");

    let thread_args = args.clone();
    let report_title = get_report_title(&args.path);
    let walk_join = crate::common::fs::threaded_walk_dir(&args.path, path_tx)?;

    let duplicate_thread = std::thread::spawn(move || {
        let mut checksum_db = crate::common::db::FileChecksumDB::new();
        match checksum_db.load(&thread_args.db) {
            Ok(()) => {}
            Err(error) => warn!(
                "cannot load checksum file {}, {error}",
                thread_args.db.display()
            ),
        }

        // Phase 1: Scan files from walker
        pb_detail.set_prefix("Scan");
        let (mut dup_db, files_scanned) =
            pipeline::scan_files(&path_rx, thread_args.min_size, &pb_detail);
        dup_db.remove_unique_size();
        let num_candidates = dup_db.m.len();
        pb_title.set_message(format!(
            "Possible Duplicates: {num_candidates}/{files_scanned} files, calculating checksums..."
        ));
        pb_detail.set_message("Checksumming files...");

        // Phase 2: Dispatch checksum work to worker pool
        let (require_checksum, require_checksum_size) =
            pipeline::dispatch_checksum_work(&dup_db, &checksum_db, &hash_tx);
        drop(hash_tx);
        let require_checksum_size_str = crate::common::util::human_size(require_checksum_size);
        pb_checksum_bar.set_length(require_checksum as u64);
        pb_title.set_message(format!(
            "{num_candidates} of {files_scanned} files are possible duplicates, calculating {require_checksum} checksums ({require_checksum_size_str})..."
        ));

        // Phase 3: Collect checksum results
        pb_detail.set_prefix("Checksum");
        pipeline::collect_checksums(
            &hash_result_rx,
            &dup_db,
            &mut checksum_db,
            &thread_args.db,
            &pipeline::CheckpointConfig {
                interval: thread_args.checksum_checkpoint_interval,
                batch_size: get_batch_size(num_candidates),
                total: require_checksum,
            },
            &pb_checksum_bar,
            &pb_detail,
        );
        pb_checksum_bar.finish_and_clear();

        // Phase 4: Select files to delete
        pb_detail.set_message("Calculating duplicates...");
        let pre_dups = db::get_duplicates(&dup_db, &checksum_db);
        let delete_files =
            pipeline::select_deletions(&pre_dups, &thread_args.delete_pattern, &mut dup_db);
        let num_delete = u64::try_from(delete_files.len()).expect("cannot convert len to u64.");

        // Phase 5: Write rmlist and delete
        pipeline::write_rmlist(&delete_files, &thread_args.rmlist);
        pb_detail.set_prefix("Deleting");
        pb_detail.set_message("Removing duplicates...");
        pb_delete_bar.set_length(num_delete);
        let delete_size = pipeline::delete_duplicates(
            &delete_files,
            thread_args.dry_run,
            thread_args.force,
            &pb_detail,
            &pb_delete_bar,
        );
        pb_delete_bar.finish_and_clear();
        let delete_size_str = crate::common::util::human_size(delete_size);

        // Phase 6: Compute final metrics and write report
        dup_db.remove_unique_size();
        let dups = db::get_duplicates(&dup_db, &checksum_db);
        let (num_dups, dup_size) = calculate_metrics(&dups);
        let summary = ReportSummary {
            files_scanned,
            num_dups,
            dup_size: crate::common::util::human_size(dup_size),
            num_delete,
            delete_size: delete_size_str,
            dry_run: thread_args.dry_run,
        };
        write_report(
            &thread_args,
            &report_title,
            &dups,
            &summary,
            &pb_title,
            &pb_detail,
        );
    });

    let hash_worker_joiner =
        crate::common::checksum::worker_pool(args.checksum_threads, hash_result_tx, &hash_rx);
    walk_join();
    hash_worker_joiner();
    duplicate_thread.join().unwrap();
    Ok(())
}

struct ReportSummary {
    files_scanned: usize,
    num_dups: u64,
    dup_size: String,
    num_delete: u64,
    delete_size: String,
    dry_run: bool,
}

fn write_report(
    args: &Args,
    report_title: &str,
    dups: &Vec<Vec<FileMetadata>>,
    summary: &ReportSummary,
    pb_title: &indicatif::ProgressBar,
    pb_detail: &indicatif::ProgressBar,
) {
    pb_detail.set_prefix("Report");
    pb_detail.set_message("Writing...");
    let deleted_text = if summary.dry_run {
        "[DRY RUN] Would have deleted"
    } else {
        "Deleted"
    };
    let ReportSummary {
        files_scanned,
        num_dups,
        dup_size,
        num_delete,
        delete_size,
        ..
    } = summary;
    pb_title.set_message(format!(
        "Scanned {files_scanned} files and found {num_dups} duplicates ({dup_size}). {deleted_text} {num_delete} files ({delete_size})."
    ));
    pb_detail.set_message("Writing report...");
    let orig = std::path::Path::new(&args.output);
    if orig.is_file() && !args.overwrite {
        pb_detail.set_message(format!(
            "Cannot save report to {output_file} because it already exists, to forcefully overwrite the file use, --overwrite=true",
            output_file = args.output
        ));
    } else if args.output.to_lowercase().ends_with(".csv") {
        match report::csv_file(&args.output, dups) {
            Ok(()) => {
                pb_title.finish_with_message(format!("Scanned {files_scanned} files and found {num_dups} duplicates ({dup_size}). {deleted_text} {num_delete} files ({delete_size}). See {output_file}", output_file = args.output));
                pb_detail.finish_and_clear();
            }
            Err(error) => pb_detail.set_message(format!(
                "cannot write report to '{output_file}', error: {error}",
                output_file = args.output
            )),
        }
    } else {
        match report::html_file(&args.output, report_title, dups) {
            Ok(()) => {
                pb_title.finish_with_message(format!("Scanned {files_scanned} files and found {num_dups} duplicates ({dup_size}). {deleted_text} {num_delete} files ({delete_size}). See {output_file}", output_file = args.output));
                pb_detail.finish_and_clear();
            }
            Err(error) => pb_detail.set_message(format!(
                "cannot write report to '{output_file}', error: {error}",
                output_file = args.output
            )),
        }
    }
}

fn calculate_metrics(dups: &[Vec<FileMetadata>]) -> (u64, u64) {
    let mut num_dups = 0u64;
    let mut dup_size = 0u64;
    for dup in dups {
        for d in dup.iter().skip(1) {
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
    (num_dups, dup_size)
}

fn get_report_title(path_list: &[std::path::PathBuf]) -> String {
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
    fn test_calculate_metrics() {
        use std::time::SystemTime;
        let t = SystemTime::UNIX_EPOCH;

        // Group of 2 identical files (1000 bytes each) → 1 extra copy, 1000 bytes wasted
        let group1 = vec![
            FileMetadata::new("/a/file1.txt", 1000, t, t),
            FileMetadata::new("/a/file2.txt", 1000, t, t),
        ];
        // Group of 3 identical files (500 bytes each) → 2 extra copies, 1000 bytes wasted
        let group2 = vec![
            FileMetadata::new("/a/file3.txt", 500, t, t),
            FileMetadata::new("/a/file4.txt", 500, t, t),
            FileMetadata::new("/a/file5.txt", 500, t, t),
        ];

        let (num_dups, dup_size) = calculate_metrics(&[group1, group2]);
        // 1 extra from group1 + 2 extras from group2 = 3 total extras
        assert_eq!(num_dups, 3);
        // 1*1000 + 2*500 = 2000 bytes wasted
        assert_eq!(dup_size, 2000);
    }

    #[test]
    fn test_get_report_title_empty() {
        assert_eq!(get_report_title(&[]), String::new());
    }

    #[test]
    fn test_get_report_title_single_path() {
        let paths = vec![std::path::PathBuf::from("/home/user/docs")];
        assert_eq!(get_report_title(&paths), "/home/user/docs");
    }

    #[test]
    fn test_get_report_title_multiple_paths() {
        let paths = vec![
            std::path::PathBuf::from("/home/user/docs"),
            std::path::PathBuf::from("/home/user/photos"),
        ];
        assert_eq!(
            get_report_title(&paths),
            "/home/user/docs,/home/user/photos"
        );
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
            checksum_checkpoint_interval: humantime::parse_duration("10s").unwrap(),
            force: true,
        };
        run(&args, Verbosity::new(0, 0)).unwrap();
    }
}

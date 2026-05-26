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

use crate::common::db::FileChecksumDB;
use crate::common::fs::FileMetadata;
use crate::duplicate::db::DuplicateFileDB;
use indicatif::ProgressBar;
use log::warn;
use std::io::Write;
use std::time::{Duration, Instant};

pub(crate) struct CheckpointConfig {
    pub(crate) interval: Duration,
    pub(crate) batch_size: usize,
    pub(crate) total: usize,
}

pub(crate) fn scan_files(
    path_rx: &crossbeam_channel::Receiver<FileMetadata>,
    min_size: u64,
    pb_detail: &ProgressBar,
) -> (DuplicateFileDB, usize) {
    let mut dup_db = DuplicateFileDB::new();
    let mut files_scanned = 0;
    for md in path_rx {
        files_scanned += 1;
        if md.size >= min_size {
            pb_detail.set_message(format!("[{files_scanned}] {}", md.path.display()));
            dup_db.put(&md);
        }
    }
    (dup_db, files_scanned)
}

pub(crate) fn dispatch_checksum_work(
    dup_db: &DuplicateFileDB,
    checksum_db: &FileChecksumDB,
    hash_tx: &crossbeam_channel::Sender<std::path::PathBuf>,
) -> (usize, u64) {
    let mut count = 0;
    let mut total_size = 0u64;
    for md in dup_db.m.values() {
        if checksum_db.get(md).is_none() {
            hash_tx.send(md.path.clone()).unwrap();
            count += 1;
            total_size += md.size;
        }
    }
    (count, total_size)
}

pub(crate) fn collect_checksums(
    hash_result_rx: &crossbeam_channel::Receiver<crate::common::checksum::FileChecksum>,
    dup_db: &DuplicateFileDB,
    checksum_db: &mut FileChecksumDB,
    db_path: &std::path::Path,
    config: &CheckpointConfig,
    pb_checksum_bar: &ProgressBar,
    pb_detail: &ProgressBar,
) {
    let mut num_hash = 0usize;
    let mut last_checkpoint_time = Instant::now();
    for hash_result in hash_result_rx {
        pb_checksum_bar.inc(1);
        pb_detail.set_message(format!("{}", hash_result.path.display()));
        let p: &std::path::Path = &hash_result.path;
        let checksum: &str = &hash_result.checksum;
        if let Some(dup_val) = dup_db.get(p) {
            checksum_db.put(dup_val, checksum);
            num_hash += 1;
            let now = Instant::now();
            let elapsed = now.duration_since(last_checkpoint_time);
            if num_hash.is_multiple_of(config.batch_size) || elapsed > config.interval {
                last_checkpoint_time = now;
                match checksum_db.write(db_path) {
                    Ok(()) => pb_checksum_bar
                        .set_message(format!("{num_hash}/{} checksums", config.total)),
                    Err(error) => warn!(
                        "cannot save checksums to db '{}', error:{error}",
                        db_path.display()
                    ),
                }
            }
        } else {
            warn!("'{}' has no entry in dup_db.", p.display());
        }
    }
    match checksum_db.write(db_path) {
        Ok(()) => {}
        Err(error) => warn!(
            "cannot save checksums to db '{}', error:{error}",
            db_path.display()
        ),
    }
    pb_checksum_bar.finish_with_message("Done");
}

pub(crate) fn compile_patterns(patterns: &[String]) -> Vec<glob::Pattern> {
    patterns
        .iter()
        .filter(|p| !p.is_empty())
        .filter_map(|p| glob::Pattern::new(p).ok())
        .collect()
}

pub(crate) fn match_file(
    path: &std::path::Path,
    delete_globs: &[glob::Pattern],
    keep_globs: &[glob::Pattern],
) -> bool {
    if delete_globs.is_empty() {
        return false;
    }
    let Some(path_str) = path.to_str() else {
        return false;
    };
    let glob_matches = |glob: &glob::Pattern| glob.matches(path_str);
    for delete_glob in delete_globs {
        if glob_matches(delete_glob) {
            if keep_globs.iter().any(glob_matches) {
                return false;
            }
            return true;
        }
    }
    false
}

pub(crate) fn select_deletions(
    pre_dups: &[Vec<FileMetadata>],
    delete_pattern: &[String],
    keep_pattern: &[String],
    dup_db: &mut DuplicateFileDB,
) -> Vec<FileMetadata> {
    let delete_globs = compile_patterns(delete_pattern);
    let keep_globs = compile_patterns(keep_pattern);
    let mut delete_files = Vec::new();
    for dup in pre_dups {
        let max_delete_per_group = dup.len() - 1;
        let mut deleted_in_group = 0;
        for md in dup {
            if deleted_in_group < max_delete_per_group
                && match_file(&md.path, &delete_globs, &keep_globs)
            {
                delete_files.push(md.clone());
                deleted_in_group += 1;
                dup_db.remove(md);
            }
        }
    }
    delete_files
}

pub(crate) fn write_rmlist(delete_files: &[FileMetadata], rmlist_path: &std::path::Path) {
    if delete_files.is_empty() {
        return;
    }
    match std::fs::File::create(rmlist_path) {
        Ok(file) => {
            let mut writer = std::io::LineWriter::new(file);
            for delete_file in delete_files {
                warn!("rmlist {}", delete_file.path.display());
                if let Some(path_str) = delete_file.path.to_str() {
                    if let Err(error) = writer.write_all(path_str.as_bytes()) {
                        warn!(
                            "cannot write line to rmlist '{}', error: {error}",
                            rmlist_path.display()
                        );
                    }
                    if let Err(error) = writer.write_all(b"\n") {
                        warn!(
                            "cannot write newline to rmlist '{}', error: {error}",
                            rmlist_path.display()
                        );
                    }
                }
            }
        }
        Err(error) => warn!(
            "cannot write rmlist '{}', error: {error}",
            rmlist_path.display()
        ),
    }
}

pub(crate) fn delete_duplicates(
    delete_files: &[FileMetadata],
    dry_run: bool,
    force: bool,
    pb_detail: &ProgressBar,
    pb_delete_bar: &ProgressBar,
) -> u64 {
    let mut delete_size = 0u64;
    for delete_file in delete_files {
        let file_path = delete_file.path.to_str().expect("cannot get file name");
        let size = crate::common::util::human_size(delete_file.size);
        pb_detail.set_message(format!("{file_path} ({size})"));
        pb_delete_bar.inc(1);
        match crate::common::fs::delete_file(file_path, dry_run, force) {
            Ok(()) => {}
            Err(error) => warn!("failed to delete file {file_path}, error: {error}"),
        }
        delete_size += delete_file.size;
        pb_delete_bar.set_message(crate::common::util::human_size(delete_size));
    }
    delete_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_files_empty_channel() {
        let (tx, rx) = crossbeam_channel::unbounded::<FileMetadata>();
        drop(tx);
        let pb = ProgressBar::hidden();
        let (db, count) = scan_files(&rx, 0, &pb);
        assert_eq!(count, 0);
        assert!(db.m.is_empty());
    }

    #[test]
    fn test_scan_files_respects_min_size() {
        let (tx, rx) = crossbeam_channel::unbounded();
        let t = std::time::SystemTime::UNIX_EPOCH;
        tx.send(FileMetadata::new("/a/large.txt", 1000, t, t))
            .unwrap();
        tx.send(FileMetadata::new("/a/small.txt", 50, t, t))
            .unwrap();
        drop(tx);
        let pb = ProgressBar::hidden();
        let (db, count) = scan_files(&rx, 100, &pb);
        assert_eq!(count, 2);
        assert!(db.m.contains_key("/a/large.txt"));
        assert!(!db.m.contains_key("/a/small.txt"));
    }

    #[test]
    fn test_scan_files_includes_all_at_or_above_min_size() {
        let (tx, rx) = crossbeam_channel::unbounded();
        let t = std::time::SystemTime::UNIX_EPOCH;
        tx.send(FileMetadata::new("/a/file1.txt", 500, t, t))
            .unwrap();
        tx.send(FileMetadata::new("/a/file2.txt", 500, t, t))
            .unwrap();
        drop(tx);
        let pb = ProgressBar::hidden();
        let (db, count) = scan_files(&rx, 0, &pb);
        assert_eq!(count, 2);
        assert_eq!(db.m.len(), 2);
    }

    #[test]
    fn test_dispatch_checksum_work_sends_unchecksummed_files() {
        let mut dup_db = DuplicateFileDB::new();
        let mut checksum_db = FileChecksumDB::new();
        let t = std::time::SystemTime::UNIX_EPOCH;
        let md1 = FileMetadata::new("/a/file1.txt", 1000, t, t);
        let md2 = FileMetadata::new("/a/file2.txt", 1000, t, t);
        dup_db.put(&md1);
        dup_db.put(&md2);
        checksum_db.put(&md2, "abc123");
        let (hash_tx, hash_rx) = crossbeam_channel::unbounded();
        let (count, size) = dispatch_checksum_work(&dup_db, &checksum_db, &hash_tx);
        drop(hash_tx);
        assert_eq!(count, 1);
        assert_eq!(size, 1000);
        let paths: Vec<String> = hash_rx
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect();
        assert_eq!(paths, vec!["/a/file1.txt"]);
    }

    #[test]
    fn test_dispatch_checksum_work_skips_already_checksummed() {
        let mut dup_db = DuplicateFileDB::new();
        let mut checksum_db = FileChecksumDB::new();
        let t = std::time::SystemTime::UNIX_EPOCH;
        let md = FileMetadata::new("/a/file.txt", 500, t, t);
        dup_db.put(&md);
        checksum_db.put(&md, "def456");
        let (hash_tx, hash_rx) = crossbeam_channel::unbounded();
        let (count, size) = dispatch_checksum_work(&dup_db, &checksum_db, &hash_tx);
        drop(hash_tx);
        assert_eq!(count, 0);
        assert_eq!(size, 0);
        assert!(hash_rx.is_empty());
    }

    #[test]
    fn test_dispatch_checksum_work_sends_all_when_no_checksums() {
        let mut dup_db = DuplicateFileDB::new();
        let checksum_db = FileChecksumDB::new();
        let t = std::time::SystemTime::UNIX_EPOCH;
        let md1 = FileMetadata::new("/a/file1.txt", 100, t, t);
        let md2 = FileMetadata::new("/a/file2.txt", 200, t, t);
        dup_db.put(&md1);
        dup_db.put(&md2);
        let (hash_tx, hash_rx) = crossbeam_channel::unbounded();
        let (count, size) = dispatch_checksum_work(&dup_db, &checksum_db, &hash_tx);
        drop(hash_tx);
        assert_eq!(count, 2);
        assert_eq!(size, 300);
        let mut paths: Vec<String> = hash_rx
            .iter()
            .map(|p| p.to_str().unwrap().to_string())
            .collect();
        paths.sort();
        assert_eq!(paths, vec!["/a/file1.txt", "/a/file2.txt"]);
    }

    #[test]
    fn test_collect_checksums_updates_db() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("checksums.txt");
        let t = std::time::SystemTime::UNIX_EPOCH;
        let md = FileMetadata::new("/a/file.txt", 100, t, t);
        let mut dup_db = DuplicateFileDB::new();
        dup_db.put(&md);
        let mut checksum_db = FileChecksumDB::new();
        let (result_tx, result_rx) = crossbeam_channel::unbounded();
        result_tx
            .send(crate::common::checksum::FileChecksum::new(
                std::path::PathBuf::from("/a/file.txt"),
                String::from("abc123"),
            ))
            .unwrap();
        drop(result_tx);
        let pb_bar = ProgressBar::hidden();
        let pb_detail = ProgressBar::hidden();
        collect_checksums(
            &result_rx,
            &dup_db,
            &mut checksum_db,
            &db_path,
            &CheckpointConfig {
                interval: Duration::from_secs(30),
                batch_size: 100,
                total: 1,
            },
            &pb_bar,
            &pb_detail,
        );
        assert_eq!(checksum_db.get(&md), Some(&String::from("abc123")));
        assert!(db_path.exists());
    }

    #[test]
    fn test_collect_checksums_ignores_unknown_paths() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("checksums.txt");
        let dup_db = DuplicateFileDB::new();
        let mut checksum_db = FileChecksumDB::new();
        let (result_tx, result_rx) = crossbeam_channel::unbounded();
        // Send a path that is NOT in dup_db
        result_tx
            .send(crate::common::checksum::FileChecksum::new(
                std::path::PathBuf::from("/unknown/file.txt"),
                String::from("xyz"),
            ))
            .unwrap();
        drop(result_tx);
        let pb_bar = ProgressBar::hidden();
        let pb_detail = ProgressBar::hidden();
        collect_checksums(
            &result_rx,
            &dup_db,
            &mut checksum_db,
            &db_path,
            &CheckpointConfig {
                interval: Duration::from_secs(30),
                batch_size: 100,
                total: 0,
            },
            &pb_bar,
            &pb_detail,
        );
        // Nothing added (path not in dup_db)
        assert_eq!(
            checksum_db.get(&FileMetadata::new(
                "/unknown/file.txt",
                0,
                std::time::SystemTime::UNIX_EPOCH,
                std::time::SystemTime::UNIX_EPOCH
            )),
            None
        );
    }

    #[test]
    fn test_collect_checksums_empty_channel() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("checksums.txt");
        let dup_db = DuplicateFileDB::new();
        let mut checksum_db = FileChecksumDB::new();
        let (result_tx, result_rx) =
            crossbeam_channel::unbounded::<crate::common::checksum::FileChecksum>();
        drop(result_tx);
        let pb_bar = ProgressBar::hidden();
        let pb_detail = ProgressBar::hidden();
        collect_checksums(
            &result_rx,
            &dup_db,
            &mut checksum_db,
            &db_path,
            &CheckpointConfig {
                interval: Duration::from_secs(30),
                batch_size: 100,
                total: 0,
            },
            &pb_bar,
            &pb_detail,
        );
        // Final write still happens (empty file is written)
        assert!(db_path.exists());
    }

    fn p(s: &str) -> glob::Pattern {
        glob::Pattern::new(s).unwrap()
    }

    #[test]
    fn test_match_file_no_patterns() {
        assert!(!match_file(
            &std::path::PathBuf::from("/a/file.txt"),
            &[],
            &[]
        ));
    }

    #[test]
    fn test_match_file_matching_pattern() {
        assert!(match_file(
            &std::path::PathBuf::from("/trash/file.txt"),
            &[p("/trash/**")],
            &[p("/important/**")],
        ));
    }

    #[test]
    fn test_match_file_matching_pattern_and_keep_pattern() {
        assert!(!match_file(
            &std::path::PathBuf::from("/trash/file.txt"),
            &[p("/trash/**")],
            &[p("**/*.txt")],
        ));
    }

    #[test]
    fn test_match_file_empty_pattern_string_ignored() {
        let empty = compile_patterns(&[String::from("")]);
        assert!(!match_file(
            &std::path::PathBuf::from("/a/file.txt"),
            &empty,
            &empty,
        ));
    }

    #[test]
    fn test_select_deletions_empty_pattern_deletes_nothing() {
        let mut dup_db = DuplicateFileDB::new();
        let t = std::time::SystemTime::UNIX_EPOCH;
        let md1 = FileMetadata::new("/a/file1.txt", 1000, t, t);
        let md2 = FileMetadata::new("/a/file2.txt", 1000, t, t);
        let dups = vec![vec![md1.clone(), md2.clone()]];
        let result = select_deletions(&dups, &[], &[], &mut dup_db);
        assert!(result.is_empty());
    }

    #[test]
    fn test_select_deletions_matches_pattern() {
        let t = std::time::SystemTime::UNIX_EPOCH;
        let md_keep = FileMetadata::new("/important/file.txt", 1000, t, t);
        let md_delete = FileMetadata::new("/trash/file.txt", 1000, t, t);
        let mut dup_db = DuplicateFileDB::new();
        dup_db.put(&md_keep);
        dup_db.put(&md_delete);
        let dups = vec![vec![md_keep.clone(), md_delete.clone()]];
        let result = select_deletions(
            &dups,
            &[String::from("/trash/**")],
            &[String::from("/important/**")],
            &mut dup_db,
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path.to_str().unwrap(), "/trash/file.txt");
        assert!(!dup_db.m.contains_key("/trash/file.txt"));
        assert!(dup_db.m.contains_key("/important/file.txt"));
    }

    #[test]
    fn test_select_deletions_keeps_at_least_one_copy() {
        let t = std::time::SystemTime::UNIX_EPOCH;
        let md1 = FileMetadata::new("/trash/file1.txt", 1000, t, t);
        let md2 = FileMetadata::new("/trash/file2.txt", 1000, t, t);
        let mut dup_db = DuplicateFileDB::new();
        dup_db.put(&md1);
        dup_db.put(&md2);
        let dups = vec![vec![md1.clone(), md2.clone()]];
        let result = select_deletions(
            &dups,
            &[String::from("/trash/**")],
            &[String::from("/important/**")],
            &mut dup_db,
        );
        // Both match the pattern but only 1 of 2 may be deleted (must keep at least 1)
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_write_rmlist_empty_list_creates_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let rmlist_path = dir.path().join("rmlist.txt");
        write_rmlist(&[], &rmlist_path);
        assert!(!rmlist_path.exists());
    }

    #[test]
    fn test_write_rmlist_creates_file_with_paths() {
        let dir = tempfile::tempdir().unwrap();
        let rmlist_path = dir.path().join("rmlist.txt");
        let t = std::time::SystemTime::UNIX_EPOCH;
        let files = vec![
            FileMetadata::new("/a/file1.txt", 100, t, t),
            FileMetadata::new("/a/file2.txt", 200, t, t),
        ];
        write_rmlist(&files, &rmlist_path);
        let content = std::fs::read_to_string(&rmlist_path).unwrap();
        assert!(content.contains("/a/file1.txt\n"));
        assert!(content.contains("/a/file2.txt\n"));
    }

    #[test]
    fn test_delete_duplicates_dry_run_preserves_files() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        std::fs::write(&file_path, b"content").unwrap();
        let t = std::time::SystemTime::UNIX_EPOCH;
        let files = vec![FileMetadata::new(file_path.to_str().unwrap(), 7, t, t)];
        let pb_detail = ProgressBar::hidden();
        let pb_delete_bar = ProgressBar::hidden();
        let size = delete_duplicates(&files, true, false, &pb_detail, &pb_delete_bar);
        assert_eq!(size, 7);
        assert!(file_path.exists());
    }

    #[test]
    fn test_delete_duplicates_removes_file_when_not_dry_run() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        std::fs::write(&file_path, b"content").unwrap();
        let t = std::time::SystemTime::UNIX_EPOCH;
        let files = vec![FileMetadata::new(file_path.to_str().unwrap(), 7, t, t)];
        let pb_detail = ProgressBar::hidden();
        let pb_delete_bar = ProgressBar::hidden();
        let size = delete_duplicates(&files, false, false, &pb_detail, &pb_delete_bar);
        assert_eq!(size, 7);
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_duplicates_empty_list_returns_zero() {
        let pb_detail = ProgressBar::hidden();
        let pb_delete_bar = ProgressBar::hidden();
        let size = delete_duplicates(&[], false, false, &pb_detail, &pb_delete_bar);
        assert_eq!(size, 0);
    }
}

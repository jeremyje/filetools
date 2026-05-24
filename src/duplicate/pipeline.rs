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
    hash_tx: crossbeam_channel::Sender<std::path::PathBuf>,
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
    checkpoint_interval: Duration,
    require_checksum: usize,
    batch_size: usize,
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
            if num_hash % batch_size == 0 || elapsed > checkpoint_interval {
                last_checkpoint_time = now;
                match checksum_db.write(db_path) {
                    Ok(()) => pb_checksum_bar
                        .set_message(format!("{num_hash}/{require_checksum} checksums")),
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
        tx.send(FileMetadata::new("/a/large.txt", 1000, t, t)).unwrap();
        tx.send(FileMetadata::new("/a/small.txt", 50, t, t)).unwrap();
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
        tx.send(FileMetadata::new("/a/file1.txt", 500, t, t)).unwrap();
        tx.send(FileMetadata::new("/a/file2.txt", 500, t, t)).unwrap();
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
        let (count, size) = dispatch_checksum_work(&dup_db, &checksum_db, hash_tx);
        assert_eq!(count, 1);
        assert_eq!(size, 1000);
        let paths: Vec<String> = hash_rx.iter().map(|p| p.to_str().unwrap().to_string()).collect();
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
        let (count, size) = dispatch_checksum_work(&dup_db, &checksum_db, hash_tx);
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
        let (count, size) = dispatch_checksum_work(&dup_db, &checksum_db, hash_tx);
        assert_eq!(count, 2);
        assert_eq!(size, 300);
        let mut paths: Vec<String> = hash_rx.iter().map(|p| p.to_str().unwrap().to_string()).collect();
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
        result_tx.send(crate::common::checksum::FileChecksum::new(
            std::path::PathBuf::from("/a/file.txt"),
            String::from("abc123"),
        )).unwrap();
        drop(result_tx);
        let pb_bar = ProgressBar::hidden();
        let pb_detail = ProgressBar::hidden();
        collect_checksums(
            &result_rx,
            &dup_db,
            &mut checksum_db,
            &db_path,
            Duration::from_secs(30),
            1,
            100,
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
        let mut dup_db = DuplicateFileDB::new();
        let mut checksum_db = FileChecksumDB::new();
        let (result_tx, result_rx) = crossbeam_channel::unbounded();
        // Send a path that is NOT in dup_db
        result_tx.send(crate::common::checksum::FileChecksum::new(
            std::path::PathBuf::from("/unknown/file.txt"),
            String::from("xyz"),
        )).unwrap();
        drop(result_tx);
        let pb_bar = ProgressBar::hidden();
        let pb_detail = ProgressBar::hidden();
        collect_checksums(
            &result_rx,
            &dup_db,
            &mut checksum_db,
            &db_path,
            Duration::from_secs(30),
            0,
            100,
            &pb_bar,
            &pb_detail,
        );
        // Nothing added (path not in dup_db)
        assert_eq!(checksum_db.get(&FileMetadata::new("/unknown/file.txt", 0, std::time::SystemTime::UNIX_EPOCH, std::time::SystemTime::UNIX_EPOCH)), None);
    }

    #[test]
    fn test_collect_checksums_empty_channel() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("checksums.txt");
        let dup_db = DuplicateFileDB::new();
        let mut checksum_db = FileChecksumDB::new();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<crate::common::checksum::FileChecksum>();
        drop(result_tx);
        let pb_bar = ProgressBar::hidden();
        let pb_detail = ProgressBar::hidden();
        collect_checksums(
            &result_rx,
            &dup_db,
            &mut checksum_db,
            &db_path,
            Duration::from_secs(30),
            0,
            100,
            &pb_bar,
            &pb_detail,
        );
        // Final write still happens (empty file is written)
        assert!(db_path.exists());
    }
}

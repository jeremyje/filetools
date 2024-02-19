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
use clap_verbosity_flag::Verbosity;
use log::warn;
use std::collections::HashMap;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// List of directories to scan for files that will have their checksums calculated.
    #[arg(long, default_value = ".")]
    pub(crate) path: Vec<std::path::PathBuf>,
    /// Output file where the checksum database will be written to.
    #[arg(long, default_value = "checksums.txt")]
    pub(crate) output: std::path::PathBuf,
    /// Number of threads for calculating checksums.
    #[arg(long, default_value_t = 2)]
    pub(crate) checksum_threads: usize,
}

pub(crate) fn run(args: &Args, verbose: &Verbosity) -> std::io::Result<()> {
    let progress_factory = crate::common::progress::ProgressFactory::new(verbose);
    let pb_title = progress_factory.create_title();
    let pb_detail = progress_factory.create_detail();
    let pb_checksum_bar = progress_factory.create_bar();

    let (path_tx, path_rx) = crossbeam_channel::unbounded();
    let (hash_tx, hash_rx) = crossbeam_channel::unbounded();
    let (hash_result_tx, hash_result_rx) =
        crossbeam_channel::unbounded::<crate::common::checksum::FileChecksum>();

    let walk_join = crate::common::fs::threaded_walk_dir(&args.path, path_tx)?;
    let hash_worker_joiner =
        crate::common::checksum::worker_pool(args.checksum_threads, hash_result_tx, &hash_rx);

    let checksum_db_filepath = std::path::PathBuf::from(&args.output);

    let checksummer_thread = std::thread::spawn(move || {
        pb_title.set_prefix("Checksum");
        pb_title.set_message("Scanning Files...");

        let mut checksum_db = crate::common::db::FileChecksumDB::new();
        match checksum_db.load(&checksum_db_filepath) {
            Ok(()) => {}
            Err(error) => {
                warn!("cannot load checksum file {checksum_db_filepath:#?}, {error}");
            }
        }
        let mut m = HashMap::new();

        pb_detail.set_prefix("Scan");
        for md in path_rx {
            if checksum_db.get(&md).is_none() {
                if let Some(path) = md.path.to_str().map(String::from) {
                    pb_detail.set_message(path.clone());
                    pb_checksum_bar.inc_length(1);
                    hash_tx.send(md.path.clone()).unwrap();
                    m.insert(path, md);
                }
            }
        }
        drop(hash_tx);

        pb_detail.set_prefix("Checksum");
        for hash_result in hash_result_rx {
            let p: &std::path::Path = &hash_result.path;

            if let Some(path) = p.to_str().map(String::from) {
                if let Some(md) = m.get(&path) {
                    pb_detail.set_message(format!("{:#?}", md.path));
                    pb_checksum_bar.inc(1);
                    checksum_db.put(md, &hash_result.checksum);
                }
            }
        }
        pb_checksum_bar.finish_and_clear();
        pb_detail.finish_and_clear();

        pb_title.set_prefix("Saving...");
        checksum_db.write(&checksum_db_filepath).unwrap();
        pb_title.set_prefix(format!("Completed, see {checksum_db_filepath:#?}"));
    });

    walk_join();
    hash_worker_joiner();
    checksummer_thread.join().unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_test() {
        let args = Args {
            path: vec![std::path::PathBuf::from(".")],
            output: std::path::PathBuf::from("checksums.txt"),
            checksum_threads: 2,
        };
        run(&args, &Verbosity::new(0, 0)).unwrap();
    }
}

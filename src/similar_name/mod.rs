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
use clap_verbosity_flag::Verbosity;
use crossbeam_channel::Receiver;
use indicatif::ProgressBar;
use log::info;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::thread;

#[derive(clap::Args, Clone)]
pub(crate) struct Args {
    /// List of directory paths to scan for similarly named files.
    #[arg(long, default_value = ".")]
    pub(crate) path: Vec<std::path::PathBuf>,
    /// Substrings that are ignored in file names to determine if a file is similar.
    /// Example: --clear-tokens=(1) will match "image.jpg" and "image (1).jpg" since the space and "(1)" are ignored.
    #[arg(long, default_value = "")]
    pub(crate) clear_tokens: Vec<String>,
    /// Minimum size of file to consider while scanning.
    #[arg(long, default_value_t = 0)]
    pub(crate) min_size: i64,
    /// Includes the size of the file when reporting.
    #[arg(long, default_value_t = false)]
    pub(crate) include_size: std::primitive::bool,
}

pub(crate) fn run(args: &Args, verbose: &Verbosity) -> io::Result<()> {
    let (path_tx, path_rx) = crossbeam_channel::unbounded();
    let walk_thread = crate::common::fs::threaded_walk_dir(args.path.as_slice(), path_tx)?;

    let progress_factory = crate::common::progress::ProgressFactory::new(verbose);
    let pb_title = progress_factory.create_title();
    let pb_detail = progress_factory.create_detail();
    pb_title.set_prefix("Similar Name");
    pb_title.set_message("Scanning...");

    let args_for_thread = args.clone();
    let pb_detail_for_thread = pb_detail.clone();
    let hash_handle = thread::spawn(move || {
        similar_name(&args_for_thread, path_rx, &pb_detail_for_thread);
    });

    walk_thread();
    hash_handle.join().unwrap();

    pb_title.set_message("Done");
    pb_detail.finish_and_clear();
    Ok(())
}

fn similar_name(args: &Args, path_rx: Receiver<FileMetadata>, pb_detail: &ProgressBar) {
    let mut files: HashMap<String, HashMap<PathBuf, FileMetadata>> = HashMap::new();
    for md in path_rx {
        let path = md.path.as_path();
        pb_detail.set_message("{path:#?}");
        if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                let reduced_name = reduce_name(file_name_str, &args.clear_tokens);
                if let Some(m) = files.get_mut(&reduced_name) {
                    m.insert(md.path.clone(), md);
                } else {
                    let mut m = HashMap::new();
                    m.insert(md.path.clone(), md);
                    files.insert(reduced_name, m);
                }
            }
        }
    }
    for (k, v) in files {
        if v.len() > 1 {
            info!("Group: {k}");
        } else {
            info!("NOT Group: {k}");
        }
    }
}

fn reduce_name(name: &str, clear_tokens: &Vec<String>) -> String {
    let mut new_name = String::from(name);
    for token in clear_tokens {
        new_name = new_name.replace(token, "");
    }
    new_name
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn similar_name() {
        let tmp_dir = tempdir().expect("create directory");
        run(
            &Args {
                path: vec![PathBuf::from(tmp_dir.path())],
                clear_tokens: vec![],
                min_size: 0,
                include_size: true,
            },
            &Verbosity::new(0, 0),
        )
        .expect("similar name");
    }

    #[test]
    fn test_reduce_name() {
        let name = reduce_name("abc.txt", &vec![String::from("ab"), String::from("txt")]);
        assert_eq!("c.", name);
    }
}

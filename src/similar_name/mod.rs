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
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::thread;

#[derive(clap::Args)]
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

pub(crate) fn run(args: &Args) -> io::Result<()> {
    let (path_tx, path_rx) = crossbeam_channel::unbounded();
    let walk_thread = crate::common::fs::threaded_walk_dir(args.path.as_slice(), path_tx)?;

    let clear_tokens = args.clear_tokens.clone();
    let hash_handle = thread::spawn(move || {
        let mut files: HashMap<String, HashMap<PathBuf, FileMetadata>> = HashMap::new();
        for md in path_rx {
            let path = md.path.as_path();
            println!("File Path: {path:#?}");
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    let reduced_name = reduce_name(file_name_str, &clear_tokens);
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
                println!("Group: {k}");
            } else {
                println!("NOT Group: {k}");
            }
        }
    });

    walk_thread();
    hash_handle.join().unwrap();
    Ok(())
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

    #[test]
    fn test_reduce_name() {
        let name = reduce_name("abc.txt", &vec![String::from("ab"), String::from("txt")]);
        assert_eq!("c.", name);
    }
}

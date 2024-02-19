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

use log::warn;
use std::fs::File;
use std::hash::Hasher;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;
use twox_hash::XxHash64;

pub(crate) fn xxhash_file(file_path: &Path) -> io::Result<String> {
    let input = File::open(file_path)?;
    let reader = BufReader::new(input);
    let digest = xxhash_buffer(reader)?;
    Ok(format!("{digest:x}"))
}

fn xxhash_buffer<R: Read>(mut reader: R) -> io::Result<u64> {
    let mut hasher = XxHash64::default();
    let mut read_buffer = [0; 1024];
    loop {
        let bytes_read = reader.read(&mut read_buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.write(&read_buffer[..bytes_read]);
    }
    Ok(hasher.finish())
}

pub(crate) struct FileChecksum {
    pub(crate) path: std::path::PathBuf,
    pub(crate) checksum: String,
}

impl FileChecksum {
    pub(crate) fn new(path: std::path::PathBuf, checksum: String) -> Self {
        Self { path, checksum }
    }
}

pub(crate) fn worker_pool(
    max_workers: usize,
    tx: crossbeam_channel::Sender<FileChecksum>,
    rx: &crossbeam_channel::Receiver<std::path::PathBuf>,
) -> impl FnOnce() {
    crate::common::thread::worker_consumer(
        "checksum",
        max_workers,
        tx,
        rx,
        move |tx: crossbeam_channel::Sender<FileChecksum>,
              rx: crossbeam_channel::Receiver<std::path::PathBuf>| {
            for p in rx {
                let hash_result = crate::common::checksum::xxhash_file(p.as_path());
                match hash_result {
                    Ok(checksum_val) => match tx.send(FileChecksum::new(p, checksum_val)) {
                        Ok(()) => {}
                        Err(error) => {
                            warn!("cannot send checksum, {error}");
                        }
                    },
                    Err(error) => warn!("Cannot compute checksum of file, '{p:#?}' {error:?}"),
                };
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xxhash_buffer() {
        let text = "abcdefg";
        let result = xxhash_buffer(text.as_bytes());
        assert_eq!(result.expect("checksum for string"), 1756566643212976685u64);
    }
}

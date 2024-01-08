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

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use super::fs::FileMetadata;

static CHECKSUM_FILE_DELIMITER: &str = "%";

/// `FileChecksumDB` tracks checksums of files.
#[derive(Debug)]
pub(crate) struct FileChecksumDB {
    m: HashMap<String, String>,
}

impl FileChecksumDB {
    /// new creates a new duplicate file DB.
    pub(crate) fn new() -> Self {
        Self { m: HashMap::new() }
    }

    /// put a `FileMetadata` record into this DB.
    pub(crate) fn put(&mut self, md: &crate::common::fs::FileMetadata, checksum: &str) {
        if let Some(key) = md.to_key() {
            let val = String::from(checksum);
            self.m.insert(key, val);
        }
    }

    pub(crate) fn get(&self, md: &FileMetadata) -> Option<&String> {
        md.to_key().and_then(|k| self.m.get(&k))
    }

    pub(crate) fn load(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        for line in std::fs::read_to_string(path)?.lines() {
            if let Some((checksum, key)) = line.split_once(CHECKSUM_FILE_DELIMITER) {
                self.m.insert(String::from(key), String::from(checksum));
            }
        }
        Ok(())
    }

    pub(crate) fn write(&self, path: &std::path::Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = std::io::LineWriter::new(file);
        for (key, checksum) in &self.m {
            let line = concat_string!(checksum, CHECKSUM_FILE_DELIMITER, key);
            writer.write_all(line.as_bytes())?;
            writer.write_all("\n".as_bytes())?;
        }
        Ok(())
    }
}

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
use std::ops::Deref;
use std::rc::Rc;

use crate::common::fs::FileMetadata;

/// DuplicateFileDB tracks potential duplicate files based on file sizes and checksums.
#[derive(Debug)]
pub(crate) struct DuplicateFileDB {
    pub(crate) m: HashMap<String, Rc<crate::common::fs::FileMetadata>>,
    //by_size: HashMap<u64, Vec<Rc<crate::common::fs::FileMetadata>>>,
    by_size: HashMap<u64, HashMap<String, Rc<crate::common::fs::FileMetadata>>>,
}

impl DuplicateFileDB {
    /// new creates a new duplicate file DB.
    pub(crate) fn new() -> Self {
        Self {
            m: HashMap::new(),
            by_size: HashMap::new(),
        }
    }

    pub(crate) fn get(
        &mut self,
        path: &std::path::Path,
    ) -> Option<&Rc<crate::common::fs::FileMetadata>> {
        path.to_str().and_then(|p| self.m.get(p))
    }

    /// put a FileMetadata record into this DB.
    pub(crate) fn put(&mut self, mdp: &crate::common::fs::FileMetadata) {
        let md = mdp.clone();
        if let Some(path) = md.path.to_str().map(String::from) {
            let size = md.size;
            let md_ptr = Rc::new(md);
            self.m.insert(path.clone(), Rc::clone(&md_ptr));

            if let Some(m) = self.by_size.get_mut(&size) {
                m.insert(path, Rc::clone(&md_ptr));
            } else {
                let mut m = HashMap::new();
                m.insert(path, Rc::clone(&md_ptr));
                self.by_size.insert(size, m);
            }
        }
    }

    /// put a FileMetadata record into this DB.
    pub(crate) fn remove(&mut self, md: &crate::common::fs::FileMetadata) {
        if let Some(path) = md.path.to_str().map(String::from) {
            self.m.remove(&path);
            if let Some(by_size_m) = self.by_size.get_mut(&md.size) {
                by_size_m.remove(&path);
            }
        }
    }

    /// remove_unique_size removes entries that have unique sizes since they are guaranteed to be unique.
    pub(crate) fn remove_unique_size(&mut self) {
        let mut remove_size: Vec<u64> = Vec::new();
        let mut remove_md = Vec::new();
        for (size, md_map) in &self.by_size {
            if md_map.len() <= 1 {
                remove_size.push(*size);
                if md_map.len() == 1 {
                    for v in md_map.values() {
                        remove_md.push(v);
                    }
                }
            }
        }
        for md in remove_md {
            let k = String::from(
                md.path
                    .to_str()
                    .expect("md.path is a Path variable that cannot be converted to a String."),
            );
            self.m.remove(&k);
        }
        for val in remove_size {
            self.by_size.remove(&val);
        }
    }
}

pub(crate) fn get_duplicates(
    dup_db: &DuplicateFileDB,
    checksum_db: &crate::common::db::FileChecksumDB,
) -> Vec<Vec<crate::common::fs::FileMetadata>> {
    let mut dups = Vec::new();

    for files_with_same_size in &dup_db.by_size {
        let mut by_checksum: HashMap<&String, Vec<FileMetadata>> = HashMap::new();
        for file in files_with_same_size.1.values() {
            if let Some(checksum) = checksum_db.get(file) {
                match by_checksum.get_mut(checksum) {
                    Some(matches) => matches.push(file.deref().clone()),
                    None => {
                        let v = vec![file.deref().clone()];
                        by_checksum.insert(checksum, v);
                    }
                }
            }
        }
        for v in by_checksum.into_values() {
            if v.len() > 1 {
                dups.push(v);
            }
        }
    }

    dups.sort();
    dups.reverse();

    dups
}

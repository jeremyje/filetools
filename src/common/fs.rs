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
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::vec::Vec;

/// Returns a parent-based de-duplication of the path list.
///
/// This means that if one path is a child of another path in the list it will be removed.
///
/// # Example
///
/// ```
/// use fs::optimize_path_list
/// let usr_bin = PathBuf::from(r"/usr/bin")
/// let usr = PathBuf::from(r"/usr")
/// let opt = PathBuf::from(r"/opt")
/// let path_list = fs::optimize_path_list(vec![&usr_bin, &opt, &opt, &usr, &usr]);
/// assert_eq!(path_list, vec![&usr, &opt])
/// ```
fn optimize_path_list<T: AsRef<Path>>(path_list: &[T]) -> Vec<&Path> {
    let mut dedup_set: HashSet<&Path> = HashSet::new();
    let mut opt_list = Vec::new();

    for p in path_list {
        dedup_set.insert(p.as_ref());
    }

    for path in path_list {
        let mut t_path = path.as_ref();
        while let Some(p) = t_path.parent() {
            if dedup_set.contains(p) {
                dedup_set.remove(path.as_ref());
            }
            t_path = p;
        }
    }

    for p in dedup_set {
        opt_list.push(p);
    }
    opt_list
}

/// Canonicalizes all the paths in a list of paths via `dunce::canonicalize`.
fn canonicalize_path_list<T: AsRef<Path>>(path_list: &[T]) -> io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    for p in path_list {
        result.push(dunce::canonicalize(p)?);
    }
    Ok(result)
}

/// Returns a canonicalized parent-based de-duplication of the path list.
///
/// This means that if one path is a child of another path in the list it will be removed.
///
/// # Example
///
/// ```
/// // Assuming env::current_dir() -> "/home/user"
/// use fs::optimize_path_list
/// let home = PathBuf::from(r"/home")
/// let pwd = PathBuf::from(r".")
/// let usr = PathBuf::from(r"/usr")
/// let usr_bin = PathBuf::from(r"/usr/bin")
/// let path_list = fs::optimize_path_list(vec![&home, &pwd, &usr, &usr_bin]);
/// assert_eq!(path_list, vec![&home, &usr])
/// ```
pub(crate) fn canonical_paths<T: AsRef<Path>>(path_list: &[T]) -> io::Result<Vec<PathBuf>> {
    let canon_list = canonicalize_path_list(path_list)?;
    let opt_list = optimize_path_list(canon_list.as_slice());
    let mut result = Vec::new();
    for p in opt_list {
        result.push(PathBuf::from(p));
    }
    Ok(result)
}

#[derive(Clone, Debug, serde::Serialize, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct FileMetadata {
    pub(crate) size: u64,
    pub(crate) path: std::path::PathBuf,
    pub(crate) created: SystemTime,
    pub(crate) modified: SystemTime,
}

impl FileMetadata {
    pub(crate) fn new(path: &str, size: u64, created: SystemTime, modified: SystemTime) -> Self {
        Self {
            size,
            path: PathBuf::from(path),
            created,
            modified,
        }
    }

    pub(crate) fn to_key(&self) -> Option<String> {
        self.path.to_str().map(|path| {
            concat_string!(
                "created://",
                time::OffsetDateTime::from(self.created).to_string(),
                "/modified://",
                time::OffsetDateTime::from(self.modified).to_string(),
                "/size://",
                self.size.to_string(),
                "/path://",
                path
            )
        })
    }
}

pub(crate) fn walk_dir(
    path_dir: &Path,
    tx: &crossbeam_channel::Sender<FileMetadata>,
) -> io::Result<()> {
    if path_dir.is_dir() {
        match fs::read_dir(path_dir) {
            Ok(entry_list) => {
                for entry in entry_list {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            match path.symlink_metadata() {
                                Ok(path_metadata) => {
                                    if path_metadata.is_dir() {
                                        walk_dir(&path, tx)?;
                                    } else if path.is_file() {
                                        if let Some(file_name) = path.to_str() {
                                            let size = path_metadata.len();
                                            let created = path_metadata.created()?;
                                            let modified = path_metadata.modified()?;

                                            tx.send(FileMetadata::new(
                                                file_name, size, created, modified,
                                            ))
                                            .map_err(|e| {
                                                io::Error::new(io::ErrorKind::Interrupted, e)
                                            })?;
                                        } else {
                                            warn!(
                                                "'{}' cannot be converted to a string, skipping.",
                                                path.display()
                                            );
                                        }
                                    }
                                }
                                Err(error) => {
                                    warn!(
                                        "cannot obtain file metadata for {}, {error}",
                                        path.display()
                                    );
                                }
                            }
                        }
                        Err(error) => warn!("cannot read directory entry, {error}"),
                    }
                }
            }
            Err(error) => warn!("directory {} cannot be listed, {error}", path_dir.display()),
        }
    }
    Ok(())
}

pub(crate) fn move_file<P: AsRef<Path>>(from: P, to: P, dry_run: bool) -> io::Result<()> {
    if !dry_run {
        fs::rename(from, to)?;
    }
    Ok(())
}

pub(crate) fn delete_file<P: AsRef<Path>>(path: P, dry_run: bool, force: bool) -> io::Result<()> {
    if !dry_run {
        match fs::remove_file(path.as_ref()) {
            Ok(result) => result,
            Err(err) => {
                if force && try_unset_read_only(&path) {
                    return fs::remove_file(path.as_ref());
                }
                return Err(err);
            }
        }
    }
    Ok(())
}

pub(crate) fn delete_directory<P: AsRef<Path>>(
    path: P,
    dry_run: bool,
    force: bool,
) -> io::Result<()> {
    if !dry_run {
        match fs::remove_dir(path.as_ref()) {
            Ok(result) => result,
            Err(err) => {
                if force && try_unset_read_only(&path) {
                    return fs::remove_dir(path.as_ref());
                }
                return Err(err);
            }
        }
    }
    Ok(())
}

#[allow(clippy::permissions_set_readonly_false)]
fn try_unset_read_only<P: AsRef<Path>>(path: &P) -> bool {
    let dir_path = PathBuf::from(path.as_ref());
    match fs::metadata(path.as_ref()) {
        Ok(metadata) => {
            let mut perms = metadata.permissions();
            perms.set_readonly(false);
            return true;
        }
        Err(err) => {
            warn!(
                "cannot change the permissions of {} to make it deletable. Err= {err}",
                dir_path.display()
            );
        }
    }
    false
}

fn scan_files<T: AsRef<Path>>(
    path_list: &[T],
    path_tx: &crossbeam_channel::Sender<crate::common::fs::FileMetadata>,
) -> std::vec::Vec<std::thread::JoinHandle<Result<(), std::io::Error>>> {
    let mut dir_walk_threads = Vec::new();
    for p in path_list {
        let path = std::path::PathBuf::from(p.as_ref());
        let tx = path_tx.clone();
        let thread_name = format!("scan_files-{}", path.display());
        let walk_handle = std::thread::Builder::new()
            .name(thread_name)
            .spawn(move || crate::common::fs::walk_dir(&path, &tx))
            .expect("failed to create scan_files thread");
        dir_walk_threads.push(walk_handle);
    }
    dir_walk_threads
}

pub(crate) fn threaded_walk_dir<T: AsRef<Path>>(
    path_list: &[T],
    path_tx: crossbeam_channel::Sender<crate::common::fs::FileMetadata>,
) -> io::Result<impl FnOnce()> {
    let paths_to_scan = canonical_paths(path_list)?;
    let thread_joiners = scan_files(paths_to_scan.as_slice(), &path_tx);

    Ok(move || {
        for thread_joiner in thread_joiners {
            thread_joiner
                .join()
                .expect("cannot join directory walk thread")
                .expect("failed to scan directory");
        }
        // Capture path_tx and drop it anyways.
        drop(path_tx);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn canonical_paths_test() {
        let _opt = PathBuf::from(r"/opt");
        let _var = PathBuf::from(r"/var");
        let _var_log = PathBuf::from(r"/var/log");
        let _etc_systemd = PathBuf::from(r"/etc/systemd");
        let input = vec![&_opt, &_var, &_var_log, &_etc_systemd];
        let mut result = canonical_paths(input.as_slice()).expect("cannot get canonical path");

        result.sort();
        assert_eq!(result, vec![_etc_systemd, _opt, _var]);
    }

    #[test]
    fn optimize_path_list_reduce_to_root() {
        let _opt = PathBuf::from(r"/opt");
        let _var = PathBuf::from(r"/var");
        let _var_log = PathBuf::from(r"/var/log");
        let _etc_systemd = PathBuf::from(r"/etc/systemd");
        let _home_root = PathBuf::from(r"/home/root");
        let _home_user = PathBuf::from(r"/home/user");
        let _root = PathBuf::from(r"/");
        let input = vec![
            &_opt,
            &_var,
            &_var_log,
            &_etc_systemd,
            &_home_root,
            &_home_user,
            &_opt,
            &_root,
        ];
        let mut result = optimize_path_list(input.as_slice());

        result.sort();
        assert_eq!(result, vec![&_root]);
    }

    #[test]
    fn optimize_path_list_test() {
        let _opt = PathBuf::from(r"/opt");
        let _var = PathBuf::from(r"/var");
        let _var_log = PathBuf::from(r"/var/log");
        let _etc_systemd = PathBuf::from(r"/etc/systemd");
        let _home_root = PathBuf::from(r"/home/root");
        let _home_user = PathBuf::from(r"/home/user");
        let _home = PathBuf::from(r"/home");
        let input = vec![
            &_opt,
            &_var,
            &_var_log,
            &_etc_systemd,
            &_home_root,
            &_home_user,
            &_opt,
            &_home,
        ];
        let mut result = optimize_path_list(input.as_slice());

        result.sort();
        assert_eq!(result, vec![&_etc_systemd, &_home, &_opt, &_var]);
    }

    #[test]
    fn optimize_path_list_disjoint() {
        let _opt = PathBuf::from(r"/opt");
        let _etc = PathBuf::from(r"/etc");
        let input = &vec![&_opt, &_etc];
        let mut result = optimize_path_list(input.as_slice());

        result.sort();
        assert_eq!(result, vec![&_etc, &_opt]);
    }

    #[test]
    fn optimize_path_list_root() {
        let _opt = PathBuf::from(r"/opt");
        let root = PathBuf::from(r"/");
        let input = &vec![&_opt, &root];
        let mut result = optimize_path_list(input.as_slice());

        result.sort();
        assert_eq!(result, vec![&root]);
    }

    #[test]
    fn optimize_path_list_empty() {
        let want: Vec<&Path> = Vec::new();
        let input: Vec<&Path> = Vec::new();
        let result = optimize_path_list(input.as_slice());

        assert_eq!(result, want);
    }

    #[test]
    fn test_delete_directory() {
        let tmp_dir = tempdir().expect("create directory");

        let readonly_dir = Path::join(tmp_dir.path(), "readonly");
        fs::create_dir_all(readonly_dir.clone()).expect("create directory");
        fs::metadata(readonly_dir.clone())
            .expect("read metadata")
            .permissions()
            .set_readonly(true);

        delete_directory(readonly_dir.clone(), true, true).expect("delete directory");
        assert_eq!(readonly_dir.exists(), true);
        delete_directory(readonly_dir.clone(), false, true).expect("delete directory");
        assert_eq!(readonly_dir.exists(), false);

        let readwrite_dir = Path::join(tmp_dir.path(), "readwrite");
        fs::create_dir_all(readwrite_dir.clone()).expect("create directory");
        fs::metadata(readwrite_dir.clone())
            .expect("read metadata")
            .permissions()
            .set_readonly(false);

        delete_directory(readwrite_dir.clone(), true, true).expect("delete directory");
        assert_eq!(readwrite_dir.exists(), true);
        delete_directory(readwrite_dir.clone(), false, true).expect("delete directory");
        assert_eq!(readwrite_dir.exists(), false);
    }
}

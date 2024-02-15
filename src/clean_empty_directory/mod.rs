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

use indicatif::ProgressBar;
use log::warn;

#[derive(clap::Args, Clone)]
pub(crate) struct Args {
    /// List of directories that will be scanned to be removed if empty.
    #[arg(long, default_value = ".")]
    pub(crate) path: Vec<std::path::PathBuf>,
    /// If false, the directories will actually be deleted. By default dry run is enabled and will only report empty directories.
    #[arg(long, default_value_t = true)]
    pub(crate) dry_run: std::primitive::bool,
}

pub(crate) fn run(args: &Args) -> std::io::Result<()> {
    let progress_factory = crate::common::progress::ProgressFactory::new();
    let mut threads = Vec::new();
    let pb_title = progress_factory.create_title();
    let pb_detail = progress_factory.create_detail();

    pb_title.set_prefix("Clean Empty Directories");
    pb_title.set_message("Scanning...");
    for path in crate::common::fs::canonical_paths(args.path.as_slice())? {
        let args = args.clone();
        let path_str = path.to_str().map_or(String::from("unknown"), String::from);

        let pb_detail = pb_detail.clone();
        let handle = std::thread::Builder::new()
            .name(format!("clean-{path_str}"))
            .spawn(
                move || match clean_empty_directory(&pb_detail, &path, args.dry_run) {
                    Ok(_) => {}
                    Err(error) => {
                        warn!("got error {error}");
                    }
                },
            )?;
        threads.push(handle);
    }
    for t in threads {
        t.join().expect("thread failed to join");
    }
    pb_detail.finish_and_clear();
    pb_title.finish_with_message("Done");
    Ok(())
}

fn clean_empty_directory(
    pb_detail: &ProgressBar,
    dir_path: &std::path::Path,
    dry_run: bool,
) -> std::io::Result<bool> {
    pb_detail.set_message(format!("{dir_path:#?}"));
    pb_detail.inc_length(1);

    let mut has_item = false;
    if dir_path.is_dir() {
        for entry in std::fs::read_dir(dir_path)? {
            let dir_entry = entry?.path();
            if dir_entry.is_file() || dir_entry.is_symlink() {
                has_item = true;
            } else if let Ok(metadata) = dir_entry.symlink_metadata() {
                if metadata.is_dir() && !metadata.is_symlink() {
                    has_item |= !clean_empty_directory(pb_detail, &dir_entry, dry_run)?;
                } else {
                    has_item = true;
                }
            } else {
                has_item = true;
            }
        }
    }
    let can_delete = !has_item;
    if can_delete {
        let dry_run_text = if dry_run { "DRY RUN" } else { "" };
        pb_detail.set_message(format!("{dry_run_text} {dir_path:#?}"));
        pb_detail.inc(1);
        crate::common::fs::delete_directory(dir_path, dry_run)?;
    }
    Ok(can_delete)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use tempdir::TempDir;

    #[test]
    fn clean_empty_directory_remove_all() {
        let tmp_dir = TempDir::new("clean_dir").expect("create directory");
        fs::create_dir_all(Path::join(tmp_dir.path(), "1/2/3/4/5/6")).expect("create directory");
        fs::create_dir_all(Path::join(tmp_dir.path(), "2/3/4/5/6")).expect("create directory");
        fs::create_dir_all(Path::join(tmp_dir.path(), "3/4/5/6")).expect("create directory");
        fs::create_dir_all(Path::join(tmp_dir.path(), "4/5/6")).expect("create directory");
        fs::create_dir_all(Path::join(tmp_dir.path(), "5/6")).expect("create directory");
        fs::create_dir_all(Path::join(tmp_dir.path(), "6")).expect("create directory");
        run(&Args {
            path: vec![PathBuf::from(tmp_dir.path())],
            dry_run: true,
        })
        .expect("clean_empty_directories");
        assert!(Path::join(tmp_dir.path(), "1/2/3/4/5/6").exists());

        run(&Args {
            path: vec![PathBuf::from(tmp_dir.path())],
            dry_run: false,
        })
        .expect("clean_empty_directories");
        assert_eq!(tmp_dir.path().exists(), false);
    }

    #[test]
    fn clean_empty_directory_remove_some() {
        let tmp_dir = TempDir::new("clean_dir").expect("create directory");
        fs::create_dir_all(Path::join(tmp_dir.path(), "1/2/3/4/5/6")).expect("create directory");
        fs::create_dir_all(Path::join(tmp_dir.path(), "2/3/4/5/6")).expect("create directory");
        fs::write(Path::join(tmp_dir.path(), "1/2/3/4.txt"), b"4").expect("write file");
        run(&Args {
            path: vec![PathBuf::from(tmp_dir.path())],
            dry_run: true,
        })
        .expect("clean_empty_directories");
        assert!(tmp_dir.path().exists());
        assert!(Path::join(tmp_dir.path(), "1/2/3/4/5/6").exists());
        assert!(Path::join(tmp_dir.path(), "2").exists());
        assert!(Path::join(tmp_dir.path(), "1/2/3/4.txt").exists());

        run(&Args {
            path: vec![PathBuf::from(tmp_dir.path())],
            dry_run: false,
        })
        .expect("clean_empty_directories");

        assert!(tmp_dir.path().exists());
        assert_eq!(Path::join(tmp_dir.path(), "1/2/3/4").exists(), false);
        assert_eq!(Path::join(tmp_dir.path(), "2").exists(), false);
        assert!(Path::join(tmp_dir.path(), "1/2/3").exists());
        assert!(Path::join(tmp_dir.path(), "1/2/3/4.txt").exists());
    }
}

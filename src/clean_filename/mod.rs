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
use log::{info, warn};
use std::io;
use std::path::Path;

#[derive(clap::Args, Clone)]
pub(crate) struct Args {
    /// List of directories that will be scanned for files to be renamed.
    #[arg(long, default_value = ".")]
    pub(crate) path: Vec<std::path::PathBuf>,
    /// When enabled (default), reports files that would be renamed without making changes. Set to false to rename files with unusual naming patterns such as URL-encoded strings.
    #[arg(long, default_value_t = true)]
    pub(crate) dry_run: std::primitive::bool,
    /// Overwrite existing files if present.
    #[arg(long, default_value_t = false)]
    pub(crate) overwrite: bool,
}

pub(crate) fn run(args: &Args, verbose: Verbosity) -> io::Result<()> {
    let (path_tx, path_rx) = crossbeam_channel::unbounded();

    let progress_factory = crate::common::progress::ProgressFactory::new(verbose);
    let pb_title = progress_factory.create_title();
    let pb_detail = progress_factory.create_detail();

    pb_title.set_prefix("Clean Filename");
    pb_title.set_message("Scanning Files...");

    let args = args.clone();
    let walk_join = crate::common::fs::threaded_walk_dir(&args.path, path_tx)?;
    let clean_filename_thread = std::thread::spawn(move || {
        let mut scan_count = 0;
        for md in path_rx {
            scan_count += 1;
            pb_detail.set_message(format!("[{scan_count}] {}", md.path.display()));
            clean_filename(&md, args.dry_run, args.overwrite);
        }
        pb_detail.set_message(format!("Scanned {scan_count} files."));
    });

    walk_join();
    clean_filename_thread.join().unwrap();
    Ok(())
}

/// Returns a cleaned version of a filename stem: URL-decoded, then stripped of
/// characters that are not alphanumeric, space, hyphen, underscore, or period.
fn clean_stem(stem: &str) -> String {
    let decoded =
        urlencoding::decode(stem).map_or_else(|_| stem.to_string(), std::borrow::Cow::into_owned);

    decoded
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
        .collect()
}

fn clean_filename(md: &crate::common::fs::FileMetadata, dry_run: bool, overwrite: bool) {
    let (stem, ext) = stem_ext(&md.path);
    let cleaned = clean_stem(&stem);

    if cleaned == stem {
        return;
    }

    if cleaned.is_empty() {
        warn!("Skipping '{}': cleaned stem is empty", md.path.display());
        return;
    }

    let parent = md.path.parent().unwrap_or(Path::new("."));
    let new_name = if ext.is_empty() {
        cleaned
    } else {
        format!("{cleaned}.{ext}")
    };
    let new_path = parent.join(&new_name);

    if new_path == md.path {
        return;
    }

    if !overwrite && new_path.exists() {
        warn!(
            "Skipping '{}': destination '{}' already exists",
            md.path.display(),
            new_path.display()
        );
        return;
    }

    if dry_run {
        info!("DRY RUN: {} => {}", md.path.display(), new_path.display());
    } else {
        match crate::common::fs::move_file(md.path.clone(), new_path.clone(), false) {
            Ok(()) => info!("{} => {}", md.path.display(), new_path.display()),
            Err(e) => warn!("Cannot rename '{}': {e}", md.path.display()),
        }
    }
}

fn stem_ext(path: &std::path::Path) -> (String, String) {
    if let Some(stem_os_str) = path.file_stem() {
        let stem = String::from(stem_os_str.to_str().expect("basename conversion"));
        if let Some(ext_os_str) = path.extension() {
            return (
                stem,
                String::from(ext_os_str.to_str().expect("extension conversion")),
            );
        }
        return (stem, String::new());
    }
    (String::new(), String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[test]
    fn test_stem_ext() {
        assert_eq!(
            stem_ext(&std::path::Path::new("test/abc.123")),
            (String::from("abc"), String::from("123"))
        );
    }

    #[test]
    fn test_clean_stem_url_encoded() {
        assert_eq!(clean_stem("hello%20world"), String::from("hello world"));
        assert_eq!(
            clean_stem("file%20name%20here"),
            String::from("file name here")
        );
        assert_eq!(clean_stem("caf%C3%A9"), String::from("café"));
    }

    #[test]
    fn test_clean_stem_removes_emojis() {
        assert_eq!(clean_stem("hello🎉world"), String::from("helloworld"));
        assert_eq!(clean_stem("photo📷2024"), String::from("photo2024"));
        assert_eq!(clean_stem("🔥hotfile🔥"), String::from("hotfile"));
    }

    #[test]
    fn test_clean_stem_removes_apostrophes() {
        assert_eq!(clean_stem("it\u{2019}s"), String::from("its"));
        assert_eq!(clean_stem("can't"), String::from("cant"));
        assert_eq!(clean_stem("O\u{2018}Brien"), String::from("OBrien"));
    }

    #[test]
    fn test_clean_stem_keeps_allowed_chars() {
        assert_eq!(
            clean_stem("my-file_name.backup"),
            String::from("my-file_name.backup")
        );
        assert_eq!(clean_stem("hello world"), String::from("hello world"));
        assert_eq!(clean_stem("café"), String::from("café"));
    }

    #[test]
    fn test_clean_stem_no_change() {
        assert_eq!(clean_stem("normal_file"), String::from("normal_file"));
    }

    #[test]
    fn test_clean_filename_url_encoded_file() {
        let tmp_dir = tempdir().expect("create directory");

        let encoded = Path::join(tmp_dir.path(), "hello%20world.txt");
        let decoded = Path::join(tmp_dir.path(), "hello world.txt");
        fs::write(&encoded, b"content").expect("write file");

        run(
            &Args {
                path: vec![PathBuf::from(tmp_dir.path())],
                dry_run: false,
                overwrite: false,
            },
            Verbosity::new(0, 0),
        )
        .expect("clean_filename");

        assert_eq!(false, encoded.exists());
        assert!(decoded.exists());
    }

    #[test]
    fn test_clean_filename_dry_run_no_rename() {
        let tmp_dir = tempdir().expect("create directory");

        let encoded = Path::join(tmp_dir.path(), "hello%20world.txt");
        fs::write(&encoded, b"content").expect("write file");

        run(
            &Args {
                path: vec![PathBuf::from(tmp_dir.path())],
                dry_run: true,
                overwrite: false,
            },
            Verbosity::new(0, 0),
        )
        .expect("clean_filename");

        assert!(encoded.exists());
    }

    #[test]
    fn test_clean_filename_no_overwrite() {
        let tmp_dir = tempdir().expect("create directory");

        let encoded = Path::join(tmp_dir.path(), "hello%20world.txt");
        let decoded = Path::join(tmp_dir.path(), "hello world.txt");
        fs::write(&encoded, b"encoded").expect("write file");
        fs::write(&decoded, b"existing").expect("write file");

        run(
            &Args {
                path: vec![PathBuf::from(tmp_dir.path())],
                dry_run: false,
                overwrite: false,
            },
            Verbosity::new(0, 0),
        )
        .expect("clean_filename");

        assert!(encoded.exists());
        assert_eq!(fs::read(&decoded).unwrap(), b"existing");
    }

    #[test]
    fn test_clean_filename_with_overwrite() {
        let tmp_dir = tempdir().expect("create directory");

        let encoded = Path::join(tmp_dir.path(), "hello%20world.txt");
        let decoded = Path::join(tmp_dir.path(), "hello world.txt");
        fs::write(&encoded, b"encoded").expect("write file");
        fs::write(&decoded, b"existing").expect("write file");

        run(
            &Args {
                path: vec![PathBuf::from(tmp_dir.path())],
                dry_run: false,
                overwrite: true,
            },
            Verbosity::new(0, 0),
        )
        .expect("clean_filename");

        assert_eq!(false, encoded.exists());
        assert_eq!(fs::read(&decoded).unwrap(), b"encoded");
    }
}

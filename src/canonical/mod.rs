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

use crate::common::fs::move_file;
use clap_verbosity_flag::Verbosity;
use crossbeam_channel::Receiver;
use indicatif::ProgressBar;
use log::info;
use log::warn;
use phf::phf_map;
use std::{
    io::Result,
    path::{Path, PathBuf},
};

use crate::common::fs::FileMetadata;

static EXTENSION_CORRECTIONS: phf::Map<&'static str, &'static str> = phf_map! {
  "jpeg" => "jpg",
  "mp4" => "m4v",
};

#[derive(clap::Args, Clone)]
pub(crate) struct Args {
    /// List of paths to scan.
    #[arg(long, default_value = ".")]
    pub(crate) path: Vec<std::path::PathBuf>,
    /// If false, will perform the file name canonicalization.
    #[arg(long, default_value_t = true)]
    pub(crate) dry_run: std::primitive::bool,
}

pub(crate) fn run(args: &Args, verbose: Verbosity) -> Result<()> {
    let (path_tx, path_rx) = crossbeam_channel::unbounded();

    let progress_factory = crate::common::progress::ProgressFactory::new(verbose);
    let pb_title = progress_factory.create_title();
    let pb_detail = progress_factory.create_detail();

    pb_title.set_prefix("Canonicalize File Names");
    pb_title.set_message("Scanning Files...");
    let args = args.clone();
    let walk_join = crate::common::fs::threaded_walk_dir(&args.path, path_tx)?;
    let duplicate_thread = std::thread::spawn(move || {
        canonicalize_filenames(&args, &pb_detail, path_rx);
    });

    walk_join();
    duplicate_thread.join().unwrap();
    Ok(())
}

fn canonicalize_filenames(args: &Args, pb_detail: &ProgressBar, path_rx: Receiver<FileMetadata>) {
    pb_detail.set_prefix("Scanning...");
    let mut files_scanned = 0;
    let mut files_renamed = 0;
    for md in path_rx {
        files_scanned += 1;
        pb_detail.set_message(format!("[{files_scanned}] {}", md.path.display()));
        if let Some(new_path) = canonicalize_path(&md.path) {
            files_renamed += 1;
            match move_file(md.path.clone(), new_path.clone(), args.dry_run) {
                Ok(()) => {
                    info!("{} => {}", md.path.display(), new_path.display());
                }
                Err(error) => {
                    warn!(
                        "Cannot rename {} => {}, Error= {error}",
                        md.path.display(),
                        new_path.display()
                    );
                }
            }
        }
    }

    pb_detail.finish_with_message(format!(
        "Scanned {files_scanned} files, renamed {files_renamed} files."
    ));
}

fn canonicalize_path(p: &Path) -> Option<PathBuf> {
    if p.is_dir() {
        return None;
    }
    if let Some(parent) = p.parent() {
        if let Some(file_stem) = p.file_stem() {
            if let Some(extension) = p.extension() {
                if let Some(new_filename) =
                    canonicalize_filename(file_stem.to_str().unwrap(), extension.to_str().unwrap())
                {
                    return Some(parent.join(new_filename));
                }
            }
        }
    }

    None
}

fn canonicalize_filename(file_stem: &str, extension: &str) -> Option<PathBuf> {
    if let Some(correction) = EXTENSION_CORRECTIONS.get(extension.to_lowercase().as_str()) {
        let mut new_file_name = file_stem.to_string();
        new_file_name.push('.');
        new_file_name.push_str(correction);
        return Some(PathBuf::from(new_file_name));
    }
    None
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn canonicalize_filenames() {
        let tmp_dir = tempdir().expect("create directory");

        let a_txt = Path::join(tmp_dir.path(), "a.txt");
        fs::write(&a_txt, b"4").expect("write file");

        let b_jpeg = Path::join(tmp_dir.path(), "b.jpeg");
        let b_jpg = Path::join(tmp_dir.path(), "b.jpg");
        fs::write(&b_jpeg, b"4").expect("write file");

        let c_jpg = Path::join(tmp_dir.path(), "c.jpg");
        fs::write(&c_jpg, b"4").expect("write file");
        run(
            &Args {
                path: vec![PathBuf::from(tmp_dir.path())],
                dry_run: true,
            },
            Verbosity::new(0, 0),
        )
        .expect("canonical");
        assert!(&a_txt.exists());
        assert!(&b_jpeg.exists());
        assert_eq!(false, (&b_jpg).exists());
        assert!(&c_jpg.exists());

        run(
            &Args {
                path: vec![PathBuf::from(tmp_dir.path())],
                dry_run: false,
            },
            Verbosity::new(1, 1),
        )
        .expect("canonical");
        assert!(&a_txt.exists());
        assert_eq!(false, (&b_jpeg).exists());
        assert!(&b_jpg.exists());
        assert!(&c_jpg.exists());
    }

    #[test]
    fn canonicalize_filename_no_correction() {
        assert_eq!(None, canonicalize_filename("abc", "txt"));
    }

    #[test]
    fn canonicalize_filename_correct_extension() {
        assert_eq!(
            Some(PathBuf::from("abc.jpg")),
            canonicalize_filename("abc", "jpeg")
        );
    }

    #[test]
    fn canonicalize_filename_no_correct_extension() {
        assert_eq!(None, canonicalize_filename("abc", "jpg"));
    }
}

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
  // Images – canonical: jpg, tif, heic, bmp
  "jpeg"  => "jpg",
  "jpe"   => "jpg",
  "jfif"  => "jpg",
  "tiff"  => "tif",
  "heif"  => "heic",
  "dib"   => "bmp",   // Device-Independent Bitmap alias
  // Video – canonical: m4v, mpg, flv, 3gp, wmv
  "mp4"   => "m4v",
  "mpeg"  => "mpg",
  "mpeg4" => "m4v",
  "m2v"   => "mpg",   // MPEG-2 video
  "3gpp"  => "3gp",   // 3GPP mobile video
  "3gp2"  => "3gp",
  "3g2"   => "3gp",
  "f4v"   => "flv",   // Flash Video variant
  "asf"   => "wmv",   // Advanced Systems Format (Windows Media container)
  // Audio – canonical: mp3, aif, wav, m4a, ogg
  "aiff"  => "aif",
  "aifc"  => "aif",   // AIFF-C compressed variant
  "wave"  => "wav",
  "m4b"   => "m4a",   // iTunes audiobook → generic MPEG-4 audio
  "m4r"   => "m4a",   // iPhone ringtone → generic MPEG-4 audio
  "oga"   => "ogg",   // Ogg audio-only container alias
  "spx"   => "ogg",   // Speex audio (also in Ogg container)
  "opus"  => "ogg",   // Opus audio (Ogg container)
  "mp2"   => "mp3",   // MPEG-1 Audio Layer II → Layer III
  // Markup / text – canonical: html, txt, md, yaml
  "htm"      => "html",
  "xhtml"    => "html",
  "shtml"    => "html",
  "text"     => "txt",
  "markdown" => "md",
  "mkd"      => "md",
  "mdown"    => "md",
  "mdwn"     => "md",
  "yml"      => "yaml",  // YAML spec recommends .yaml
  // Archives – preserve the tar layer in the extension
  "tgz"  => "tar.gz",
  "taz"  => "tar.gz",
  "tbz2" => "tar.bz2",
  "tbz"  => "tar.bz2",
  "txz"  => "tar.xz",
  "tzst" => "tar.zst",
};

#[derive(clap::Args, Clone)]
pub(crate) struct Args {
    /// List of paths to scan for files with non-canonical extensions (e.g. .jpeg → .jpg, .mp4 → .m4v).
    #[arg(long, default_value = ".")]
    pub(crate) path: Vec<std::path::PathBuf>,
    /// When enabled (default), reports files that would be renamed without making changes. Set to false to actually rename files.
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
    let ext_lower = extension.to_lowercase();
    let new_ext = if let Some(&correction) = EXTENSION_CORRECTIONS.get(ext_lower.as_str()) {
        correction.to_string()
    } else if ext_lower != extension {
        ext_lower
    } else {
        return None;
    };
    // Avoid double-tar: "archive.tar.tgz" → "archive.tar.gz", not "archive.tar.tar.gz"
    let stem = if new_ext.starts_with("tar.")
        && file_stem.to_lowercase().ends_with(".tar")
    {
        &file_stem[..file_stem.len() - 4]
    } else {
        file_stem
    };
    Some(PathBuf::from(format!("{stem}.{new_ext}")))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn canonicalize_filenames_dry_run_then_rename() {
        let tmp_dir = tempdir().expect("create directory");

        let a_txt = Path::join(tmp_dir.path(), "a.txt");
        fs::write(&a_txt, b"4").expect("write file");
        let b_jpeg = Path::join(tmp_dir.path(), "b.jpeg");
        let b_jpg = Path::join(tmp_dir.path(), "b.jpg");
        fs::write(&b_jpeg, b"4").expect("write file");
        let c_jpg = Path::join(tmp_dir.path(), "c.jpg");
        fs::write(&c_jpg, b"4").expect("write file");

        // dry_run=true: nothing should change
        run(
            &Args {
                path: vec![PathBuf::from(tmp_dir.path())],
                dry_run: true,
            },
            Verbosity::new(0, 0),
        )
        .expect("canonical");
        assert!(a_txt.exists());
        assert!(b_jpeg.exists());
        assert!(!b_jpg.exists());
        assert!(c_jpg.exists());

        // dry_run=false: b.jpeg → b.jpg
        run(
            &Args {
                path: vec![PathBuf::from(tmp_dir.path())],
                dry_run: false,
            },
            Verbosity::new(1, 1),
        )
        .expect("canonical");
        assert!(a_txt.exists());
        assert!(!b_jpeg.exists());
        assert!(b_jpg.exists());
        assert!(c_jpg.exists());
    }

    #[test]
    fn canonicalize_filenames_uppercase() {
        let tmp_dir = tempdir().expect("create directory");
        let a_upper = Path::join(tmp_dir.path(), "a.JPG");
        let a_lower = Path::join(tmp_dir.path(), "a.jpg");
        fs::write(&a_upper, b"img").expect("write file");

        run(
            &Args {
                path: vec![PathBuf::from(tmp_dir.path())],
                dry_run: false,
            },
            Verbosity::new(0, 0),
        )
        .expect("canonical");
        assert!(!a_upper.exists());
        assert!(a_lower.exists());
    }

    #[test]
    fn canonicalize_filename_table() {
        // (stem, extension, expected output)
        // None means no rename needed.
        let cases: &[(&str, &str, Option<&str>)] = &[
            // Already canonical – no change
            ("abc", "txt",  None),
            ("abc", "jpg",  None),
            ("abc", "mp3",  None),
            // Alias corrections – images
            ("abc", "jpeg",     Some("abc.jpg")),
            ("abc", "jpe",      Some("abc.jpg")),
            ("abc", "jfif",     Some("abc.jpg")),
            ("abc", "tiff",     Some("abc.tif")),
            ("abc", "heif",     Some("abc.heic")),
            ("abc", "dib",      Some("abc.bmp")),
            // Alias corrections – video
            ("abc", "mp4",      Some("abc.m4v")),
            ("abc", "mpeg",     Some("abc.mpg")),
            ("abc", "mpeg4",    Some("abc.m4v")),
            ("abc", "m2v",      Some("abc.mpg")),
            ("abc", "3gpp",     Some("abc.3gp")),
            ("abc", "3gp2",     Some("abc.3gp")),
            ("abc", "3g2",      Some("abc.3gp")),
            ("abc", "f4v",      Some("abc.flv")),
            ("abc", "asf",      Some("abc.wmv")),
            // Alias corrections – audio
            ("abc", "aiff",     Some("abc.aif")),
            ("abc", "aifc",     Some("abc.aif")),
            ("abc", "wave",     Some("abc.wav")),
            ("abc", "m4b",      Some("abc.m4a")),
            ("abc", "m4r",      Some("abc.m4a")),
            ("abc", "oga",      Some("abc.ogg")),
            ("abc", "spx",      Some("abc.ogg")),
            ("abc", "opus",     Some("abc.ogg")),
            ("abc", "mp2",      Some("abc.mp3")),
            // Alias corrections – markup / text
            ("abc", "htm",      Some("abc.html")),
            ("abc", "xhtml",    Some("abc.html")),
            ("abc", "shtml",    Some("abc.html")),
            ("abc", "text",     Some("abc.txt")),
            ("abc", "markdown", Some("abc.md")),
            ("abc", "mkd",      Some("abc.md")),
            ("abc", "mdown",    Some("abc.md")),
            ("abc", "mdwn",     Some("abc.md")),
            ("abc", "yml",      Some("abc.yaml")),
            // Alias corrections – archives
            ("abc", "tgz",      Some("abc.tar.gz")),
            ("abc", "taz",      Some("abc.tar.gz")),
            ("abc", "tbz2",     Some("abc.tar.bz2")),
            ("abc", "tbz",      Some("abc.tar.bz2")),
            ("abc", "txz",      Some("abc.tar.xz")),
            ("abc", "tzst",     Some("abc.tar.zst")),
            // Uppercase extension → lowercase (no alias needed)
            ("abc", "JPG",      Some("abc.jpg")),
            ("abc", "PNG",      Some("abc.png")),
            ("abc", "MP4",      Some("abc.m4v")),
            ("abc", "JPEG",     Some("abc.jpg")),
            // Double-tar prevention: stem already ends with .tar
            ("archive.tar", "tgz",  Some("archive.tar.gz")),
            ("archive.tar", "tbz2", Some("archive.tar.bz2")),
            ("archive.tar", "txz",  Some("archive.tar.xz")),
            ("archive.TAR", "tgz",  Some("archive.tar.gz")),
            // Non-.tar stem is unaffected
            ("archive",     "tgz",  Some("archive.tar.gz")),
        ];

        for &(stem, ext, expected) in cases {
            assert_eq!(
                expected.map(PathBuf::from),
                canonicalize_filename(stem, ext),
                "stem={stem:?} ext={ext:?}"
            );
        }
    }
}

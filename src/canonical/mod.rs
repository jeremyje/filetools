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
  // Audio – canonical: mp3, aif, wav, m4a, ogg, flac, aac
  "aiff"  => "aif",
  "aifc"  => "aif",   // AIFF-C compressed variant
  "wave"  => "wav",
  "m4b"   => "m4a",   // iTunes audiobook → generic MPEG-4 audio
  "m4r"   => "m4a",   // iPhone ringtone → generic MPEG-4 audio
  "oga"   => "ogg",   // Ogg audio-only container alias
  "spx"   => "ogg",   // Speex audio (also in Ogg container)
  "opus"  => "ogg",   // Opus audio (Ogg container)
  "mp2"   => "mp3",   // MPEG-1 Audio Layer II → Layer III
  // Markup / text – canonical: html, txt, md, yaml, xml
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
  "tgz"      => "tar.gz",
  "taz"      => "tar.gz",
  "tbz2"     => "tar.bz2",
  "tbz"      => "tar.bz2",
  "txz"      => "tar.xz",
  "tzst"     => "tar.zst",
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

    #[test]
    fn canonicalize_filename_uppercase_to_lowercase() {
        assert_eq!(
            Some(PathBuf::from("abc.jpg")),
            canonicalize_filename("abc", "JPG")
        );
        assert_eq!(
            Some(PathBuf::from("abc.jpg")),
            canonicalize_filename("abc", "JPEG")
        );
        assert_eq!(
            Some(PathBuf::from("abc.png")),
            canonicalize_filename("abc", "PNG")
        );
        assert_eq!(
            Some(PathBuf::from("abc.m4v")),
            canonicalize_filename("abc", "MP4")
        );
    }

    #[test]
    fn canonicalize_filename_extended_corrections() {
        // Images
        assert_eq!(
            Some(PathBuf::from("abc.jpg")),
            canonicalize_filename("abc", "jpe")
        );
        assert_eq!(
            Some(PathBuf::from("abc.jpg")),
            canonicalize_filename("abc", "jfif")
        );
        assert_eq!(
            Some(PathBuf::from("abc.tif")),
            canonicalize_filename("abc", "tiff")
        );
        assert_eq!(
            Some(PathBuf::from("abc.heic")),
            canonicalize_filename("abc", "heif")
        );
        assert_eq!(
            Some(PathBuf::from("abc.bmp")),
            canonicalize_filename("abc", "dib")
        );
        // Video
        assert_eq!(
            Some(PathBuf::from("abc.mpg")),
            canonicalize_filename("abc", "mpeg")
        );
        assert_eq!(
            Some(PathBuf::from("abc.mpg")),
            canonicalize_filename("abc", "m2v")
        );
        assert_eq!(
            Some(PathBuf::from("abc.3gp")),
            canonicalize_filename("abc", "3gpp")
        );
        assert_eq!(
            Some(PathBuf::from("abc.3gp")),
            canonicalize_filename("abc", "3g2")
        );
        assert_eq!(
            Some(PathBuf::from("abc.flv")),
            canonicalize_filename("abc", "f4v")
        );
        assert_eq!(
            Some(PathBuf::from("abc.wmv")),
            canonicalize_filename("abc", "asf")
        );
        // Audio
        assert_eq!(
            Some(PathBuf::from("abc.aif")),
            canonicalize_filename("abc", "aiff")
        );
        assert_eq!(
            Some(PathBuf::from("abc.aif")),
            canonicalize_filename("abc", "aifc")
        );
        assert_eq!(
            Some(PathBuf::from("abc.wav")),
            canonicalize_filename("abc", "wave")
        );
        assert_eq!(
            Some(PathBuf::from("abc.m4a")),
            canonicalize_filename("abc", "m4b")
        );
        assert_eq!(
            Some(PathBuf::from("abc.m4a")),
            canonicalize_filename("abc", "m4r")
        );
        assert_eq!(
            Some(PathBuf::from("abc.ogg")),
            canonicalize_filename("abc", "oga")
        );
        assert_eq!(
            Some(PathBuf::from("abc.mp3")),
            canonicalize_filename("abc", "mp2")
        );
        // Markup / text
        assert_eq!(
            Some(PathBuf::from("abc.html")),
            canonicalize_filename("abc", "htm")
        );
        assert_eq!(
            Some(PathBuf::from("abc.html")),
            canonicalize_filename("abc", "xhtml")
        );
        assert_eq!(
            Some(PathBuf::from("abc.txt")),
            canonicalize_filename("abc", "text")
        );
        assert_eq!(
            Some(PathBuf::from("abc.md")),
            canonicalize_filename("abc", "markdown")
        );
        assert_eq!(
            Some(PathBuf::from("abc.md")),
            canonicalize_filename("abc", "mkd")
        );
        assert_eq!(
            Some(PathBuf::from("abc.yaml")),
            canonicalize_filename("abc", "yml")
        );
        // Archives
        assert_eq!(
            Some(PathBuf::from("abc.tar.gz")),
            canonicalize_filename("abc", "tgz")
        );
        assert_eq!(
            Some(PathBuf::from("abc.tar.gz")),
            canonicalize_filename("abc", "taz")
        );
        assert_eq!(
            Some(PathBuf::from("abc.tar.bz2")),
            canonicalize_filename("abc", "tbz2")
        );
        assert_eq!(
            Some(PathBuf::from("abc.tar.bz2")),
            canonicalize_filename("abc", "tbz")
        );
        assert_eq!(
            Some(PathBuf::from("abc.tar.xz")),
            canonicalize_filename("abc", "txz")
        );
        assert_eq!(
            Some(PathBuf::from("abc.tar.zst")),
            canonicalize_filename("abc", "tzst")
        );
    }

    #[test]
    fn canonicalize_filename_no_double_tar() {
        // stem already contains .tar — must not produce .tar.tar.*
        assert_eq!(
            Some(PathBuf::from("archive.tar.gz")),
            canonicalize_filename("archive.tar", "tgz")
        );
        assert_eq!(
            Some(PathBuf::from("archive.tar.bz2")),
            canonicalize_filename("archive.tar", "tbz2")
        );
        assert_eq!(
            Some(PathBuf::from("archive.tar.xz")),
            canonicalize_filename("archive.tar", "txz")
        );
        // Case-insensitive stem check: .TAR is stripped too
        assert_eq!(
            Some(PathBuf::from("archive.tar.gz")),
            canonicalize_filename("archive.TAR", "tgz")
        );
        // Normal case unaffected
        assert_eq!(
            Some(PathBuf::from("archive.tar.gz")),
            canonicalize_filename("archive", "tgz")
        );
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
        assert_eq!(false, a_upper.exists());
        assert!(a_lower.exists());
    }
}

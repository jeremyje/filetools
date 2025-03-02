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

use log::{trace, warn};
use std::io::{self, Write};
use clap_verbosity_flag::Verbosity;

#[derive(clap::Args, Clone)]
pub(crate) struct Args {
    /// List of directories that will be scanned for files to be renamed.
    #[arg(long, default_value = ".")]
    pub(crate) path: Vec<std::path::PathBuf>,
    /// If false, filenames that contain usual naming parts such as URL encoded strings are cleaned up. By default dry run is enabled and will only report the rename attempts.
    #[arg(long, default_value_t = true)]
    pub(crate) dry_run: std::primitive::bool,
    /// Overwrite existing files if present.
    #[arg(long, default_value_t = false)]
    pub(crate) overwrite: bool,
}

pub(crate) fn run(args: &Args, verbose: &Verbosity) -> io::Result<()> {
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
        let mut error_count =0;
        for md in path_rx {
            scan_count += 1;
            pb_detail.set_message(format!("[{scan_count}] {:#?}", md.path));
            let filename = md.path.clone();
            match clean_filename(&md) {
                Ok(()) => {

                }
                Err(error) => {
                    pb_detail.set_message("cannot sanitize filename {filename} {error}.");
                    error_count +=1;
                }
            }
        }

        pb_detail.set_message("Scanned {scan_count} files, {error_count} errors.");
    });

    walk_join();
    clean_filename_thread.join().unwrap();
    Ok(())
}

fn clean_filename(md: &crate::common::fs::FileMetadata) -> io::Result<()> {
    let (stem, ext) = stem_ext(&md.path);

    trace!("{stem}.{ext}");

    Ok(())
}

fn stem_ext(path: &std::path::Path) -> (String, String) {
    if let Some(stem_os_str) = path.file_stem() {
        let stem = String::from(stem_os_str.to_str().expect("basename conversion"));
        if let Some(ext_os_str) = path.extension() {
            return (stem, String::from(ext_os_str.to_str().expect("extension conversion")))
        } else {
            return (stem, String::new())
        }
    }
    return (String::new(), String::new())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stem_ext() {
        assert_eq!(stem_ext(&std::path::Path::new("test/abc.123")), (String::from("abc"), String::from("123")));
    }
}

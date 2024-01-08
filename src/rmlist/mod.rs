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

use std::io;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// List of files that contain 1 file name per line of files to delete.
    #[arg(long, default_value = ".")]
    pub(crate) path: Vec<std::path::PathBuf>,
    /// If false, will perform the delete based on the pattern filtering provided by --delete_pattern.
    #[arg(long, default_value_t = true)]
    pub(crate) dry_run: std::primitive::bool,
}

pub(crate) fn run(args: &Args) -> io::Result<()> {
    let progress_factory = crate::common::progress::ProgressFactory::new();
    let pb_title = progress_factory.create_title();
    let pb_detail = progress_factory.create_detail();

    pb_title.set_prefix("rmlist");
    pb_title.set_message("Deleting Files...");

    if args.dry_run {
        pb_detail.set_prefix("[DRY RUN] Deleting");
    } else {
        pb_detail.set_prefix("Deleting");
    }

    let mut num_files = 0;
    for dir_path in &args.path {
        for line in std::fs::read_to_string(dir_path)?.lines() {
            match crate::common::fs::delete_file(line, args.dry_run) {
                Ok(()) => {
                    pb_detail.set_message(line.to_string());
                    num_files += 1;
                }
                Err(error) => {
                    pb_detail.set_message(format!("cannot delete {line}, {error}"));
                }
            }
        }
    }

    pb_detail.finish_and_clear();
    pb_title.finish_with_message(format!("Deleted {num_files} files."));

    Ok(())
}

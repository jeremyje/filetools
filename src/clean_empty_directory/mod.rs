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

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::warn;

#[derive(clap::Args)]
pub struct Args {
    /// List of directories that will be scanned to be removed if empty.
    #[arg(long, default_value = ".")]
    pub path: Vec<std::path::PathBuf>,
    /// If false, the directories will actually be deleted. By default dry run is enabled and will only report empty directories.
    #[arg(long, default_value_t = true)]
    pub dry_run: std::primitive::bool,
}

fn new_progress_bar(multi_progress: &MultiProgress, spinner_style: &ProgressStyle) -> ProgressBar {
    let progress_bar = multi_progress.add(ProgressBar::new(1));
    progress_bar.set_style(spinner_style.clone());
    progress_bar
}

pub fn run(args: &Args) -> std::io::Result<()> {
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let multi_progress = MultiProgress::new();
    let mut threads = Vec::new();
    for path in crate::common::fs::canonical_paths(args.path.as_slice())? {
        let dry_run = args.dry_run;
        let path_str = path
            .to_str()
            .map(String::from)
            .unwrap_or(String::from("unknown"));
        let progress_bar = new_progress_bar(&multi_progress, &spinner_style);
        let action_progress_bar = new_progress_bar(&multi_progress, &spinner_style);

        let handle = std::thread::Builder::new()
            .name(format!("clean-{path_str}"))
            .spawn(move || {
                match clean_empty_directory_with_progress(
                    &progress_bar,
                    &action_progress_bar,
                    &path,
                    dry_run,
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        warn!("got error {error}");
                    }
                }
                progress_bar.finish_with_message("done");
                action_progress_bar.finish_with_message("done");
            })?;
        threads.push(handle);
    }
    for t in threads {
        t.join().expect("thread failed to join");
    }
    Ok(())
}

fn clean_empty_directory_with_progress(
    progress_bar: &ProgressBar,
    action_progress_bar: &ProgressBar,
    dir_path: &std::path::Path,
    dry_run: bool,
) -> std::io::Result<bool> {
    progress_bar.set_message(format!("{dir_path:#?}"));
    progress_bar.inc(1);

    let mut has_item = false;
    if dir_path.is_dir() {
        for entry in std::fs::read_dir(dir_path)? {
            let dir_entry = entry?.path();
            if dir_entry.is_file() || dir_entry.is_symlink() {
                has_item = true;
            } else if let Ok(metadata) = dir_entry.symlink_metadata() {
                if metadata.is_dir() && !metadata.is_symlink() {
                    has_item |= !clean_empty_directory_with_progress(
                        progress_bar,
                        action_progress_bar,
                        &dir_entry,
                        dry_run,
                    )?;
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
        action_progress_bar.set_message(format!("{dry_run_text} {dir_path:#?}"));
        action_progress_bar.inc(1);
        crate::common::fs::delete_directory(dir_path, dry_run)?;
    }
    Ok(can_delete)
}

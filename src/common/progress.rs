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
use indicatif_log_bridge::LogWrapper;
use log::warn;

/// Creates a standard multi `ProgressBar` for filetool.
pub(crate) struct ProgressFactory {
    title_style: ProgressStyle,
    detail_style: ProgressStyle,
    bar_style: ProgressStyle,
    danger_bar_style: ProgressStyle,
    multi_progress: MultiProgress,
}

impl ProgressFactory {
    /// new creates a new duplicate file DB.
    pub(crate) fn new() -> Self {
        let multi_progress = MultiProgress::new();
        let logger =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .build();
        let log_wrapper = LogWrapper::new(multi_progress.clone(), logger);
        match log_wrapper.try_init() {
            Ok(()) => {}
            Err(err) => {
                warn!("Cannot set global logger to indicatif-log-bridge. Error= {err}");
            }
        }
        Self {
            title_style: ProgressStyle::with_template(
                "[{elapsed_precise}] {prefix:.bold.dim} {wide_msg}",
            )
            .unwrap(),
            detail_style: ProgressStyle::with_template("{prefix:.bold.dim} {wide_msg}").unwrap(),
            bar_style: ProgressStyle::with_template("{wide_bar:60.blue/white} {msg} [{eta}]")
                .unwrap(),
            danger_bar_style: ProgressStyle::with_template("{wide_bar:60.red/white} {msg}")
                .unwrap(),
            multi_progress,
        }
    }

    pub(crate) fn create_title(&self) -> ProgressBar {
        let progress_bar = self.multi_progress.add(ProgressBar::new(1));
        progress_bar.set_style(self.title_style.clone());
        progress_bar.enable_steady_tick(std::time::Duration::from_secs(1));
        progress_bar
    }

    pub(crate) fn create_detail(&self) -> ProgressBar {
        let progress_bar = self.multi_progress.add(ProgressBar::new(1));
        progress_bar.set_style(self.detail_style.clone());
        progress_bar
    }

    pub(crate) fn create_bar(&self) -> ProgressBar {
        let progress_bar = self.multi_progress.add(ProgressBar::new(1));
        progress_bar.set_style(self.bar_style.clone());
        progress_bar
    }

    pub(crate) fn create_danger(&self) -> ProgressBar {
        let progress_bar = self.multi_progress.add(ProgressBar::new(1));
        progress_bar.set_style(self.danger_bar_style.clone());
        progress_bar
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_progress_bars() {
        let factory = ProgressFactory::new();
        let pb_title = factory.create_title();
        let pb_detail = factory.create_detail();
        let pb_bar = factory.create_bar();
        pb_title.set_length(100);
        pb_title.inc(1);
        pb_detail.inc(1);
        pb_bar.set_length(100);
        pb_bar.inc(500);
        pb_title.finish();
        assert!(pb_title.is_finished());
        assert!(!pb_bar.is_finished());
        assert!(!pb_detail.is_finished());
    }
}

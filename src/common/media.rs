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

#![allow(unused)]

use mp4::Metadata;
use std::fs::File;
use std::io;
use std::time::SystemTime;

#[derive(Debug)]
struct MediaMetadata {
    duration: std::time::Duration,
    title: Option<String>,
    summary: Option<String>,
}

impl MediaMetadata {
    fn new(duration: std::time::Duration, title: Option<String>, summary: Option<String>) -> Self {
        Self {
            duration,
            title,
            summary,
        }
    }
}

fn try_mp4(path: &std::path::Path) -> io::Result<MediaMetadata> {
    let f = File::open(path).unwrap();
    let size = f.metadata()?.len();
    try_mp4_reader(f, size)
}

fn try_mp4_file(f: &File) -> io::Result<MediaMetadata> {
    let size = f.metadata()?.len();
    try_mp4_reader(f, size)
}

fn try_mp4_content(file_content: &[u8]) -> io::Result<MediaMetadata> {
    let size: u64 = file_content.len().try_into().unwrap();
    let cursor = std::io::Cursor::new(file_content);
    try_mp4_reader(cursor, size)
}

fn try_mp4_reader<R: std::io::Read + std::io::Seek>(
    reader: R,
    size: u64,
) -> io::Result<MediaMetadata> {
    let mp4md = mp4::Mp4Reader::read_header(reader, size).unwrap();
    let duration = mp4md.duration();
    let md4 = mp4md.metadata();

    let mut opt_title = Option::None;
    if let Some(title) = md4.title() {
        opt_title = Option::from(title.to_string());
    }
    let mut opt_summary = Option::None;
    if let Some(summary) = md4.summary() {
        opt_summary = Option::from(summary.to_string());
    }
    let md = MediaMetadata::new(duration, opt_title, opt_summary);
    Ok(md)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_mp4_content() {
        //let mp4_file = testfiles::get().get_file("path").unwrap();
        let mp4_file = testfiles::sample_video_mp4();
        let md = try_mp4_content(mp4_file).unwrap();
        assert_eq!(md.title, Option::None);
        assert_eq!(md.duration, std::time::Duration::from_millis(5054));
    }

    #[test]
    fn test_try_mp4_content_from_file() {
        let mp4_file = testfiles::get()
            .get_file("video/sample-from-clipchamp-720p.mp4")
            .unwrap();
        let md = try_mp4_content(mp4_file.contents()).unwrap();
        assert_eq!(md.title, Option::None);
        assert_eq!(md.duration, std::time::Duration::from_millis(5054));
    }

    #[test]
    fn test_filemetadata_to_key() {
        let rfc2822 =
            chrono::DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0200").unwrap();
        let md = crate::common::fs::FileMetadata::new(
            "file.txt",
            50,
            SystemTime::from(rfc2822),
            SystemTime::from(rfc2822),
        );
        assert_eq!(md.to_key(), "created://2003-07-01 8:52:37.0 +00:00:00/modified://2003-07-01 8:52:37.0 +00:00:00/size://50/path://file.txt");
    }
}

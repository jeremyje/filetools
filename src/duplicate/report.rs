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

use csv::Writer;
use handlebars::Handlebars;
use serde_json::json;
use std::{fs, vec::Vec};

use crate::common::fs::FileMetadata;

const DUPLICATE_REPORT_HTML: &str = include_str!("embed/templates/duplicate-report.html");

pub(crate) fn csv_file(
    output_path: &str,
    dups: &Vec<Vec<crate::common::fs::FileMetadata>>,
) -> std::io::Result<()> {
    let mut wtr = Writer::from_path(output_path)?;
    for dup in dups {
        let row: Vec<&str> = dup
            .iter()
            .map(|md: &FileMetadata| md.path.to_str().unwrap_or(""))
            .collect();
        wtr.write_record(row)?;
    }
    wtr.flush()?;
    Ok(())
}

pub(crate) fn html_file(
    output_path: &str,
    title: &str,
    dups: &Vec<Vec<crate::common::fs::FileMetadata>>,
    timestamp: &str,
    duration: &str,
) -> std::io::Result<()> {
    let contents = html(title, dups, timestamp, duration).unwrap();
    fs::write(output_path, contents)
}

pub(crate) fn html(
    title: &str,
    dups: &Vec<Vec<crate::common::fs::FileMetadata>>,
    timestamp: &str,
    duration: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut reg = Handlebars::new();
    reg.set_strict_mode(true);
    reg.register_helper("humansize", Box::new(humansize));
    reg.register_template_string("duplicate-report", DUPLICATE_REPORT_HTML)?;
    let num_groups = dups.len();
    Ok(reg.render(
        "duplicate-report",
        &json!({
            "title": title,
            "groups": dups,
            "timestamp": timestamp,
            "duration": duration,
            "num_groups": num_groups,
        }),
    )?)
}

// define a custom helper
fn humansize(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> Result<(), handlebars::RenderError> {
    // get parameter from helper or throw an error
    let param = h
        .param(0)
        .ok_or(handlebars::RenderErrorReason::ParamNotFoundForIndex(
            "humansize",
            0,
        ))?;
    let val = param.value();
    if !val.is_null() {
        let val = param.value().as_u64().unwrap();
        let rendered = crate::common::util::human_size(val);
        write!(out, "{rendered}")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn make_group(paths_and_sizes: &[(&str, u64)]) -> Vec<FileMetadata> {
        let t = SystemTime::UNIX_EPOCH;
        paths_and_sizes
            .iter()
            .map(|(path, size)| FileMetadata::new(path, *size, t, t))
            .collect()
    }

    #[test]
    fn test_empty_html() {
        let dups = Vec::new();
        let output = html("Duplicate Title String", &dups, "", "").unwrap();
        assert!(output.contains("Duplicate Title String"));
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("0 duplicate groups"));
        assert!(!output.contains("{{"));
    }

    #[test]
    fn test_html_with_timestamp_and_duration() {
        let dups = Vec::new();
        let output = html("My Title", &dups, "2024-01-15 14:30:45 -0700", "1m 23s").unwrap();
        assert!(output.contains("2024-01-15 14:30:45 -0700"));
        assert!(output.contains("1m 23s"));
    }

    #[test]
    fn test_html_without_timestamp_omits_timing_section() {
        let dups = Vec::new();
        let output = html("My Title", &dups, "", "").unwrap();
        assert!(!output.contains("Generated:"));
        assert!(!output.contains("Duration:"));
        assert!(!output.contains("Report generated on"));
    }

    #[test]
    fn test_html_group_count_single_group() {
        let dups = vec![make_group(&[("/a/file.txt", 1000), ("/b/file.txt", 1000)])];
        let output = html("Title", &dups, "", "").unwrap();
        assert!(output.contains("1 duplicate groups"));
    }

    #[test]
    fn test_html_group_count_multiple_groups() {
        let dups = vec![
            make_group(&[("/a/file1.txt", 500), ("/b/file1.txt", 500)]),
            make_group(&[("/c/photo.jpg", 2048), ("/d/photo.jpg", 2048)]),
            make_group(&[("/e/doc.pdf", 4096), ("/f/doc.pdf", 4096)]),
        ];
        let output = html("Title", &dups, "", "").unwrap();
        assert!(output.contains("3 duplicate groups"));
    }

    #[test]
    fn test_html_renders_file_paths() {
        let dups = vec![make_group(&[
            ("/home/user/photos/vacation.jpg", 1_500_000),
            ("/backup/photos/vacation.jpg", 1_500_000),
        ])];
        let output = html("Title", &dups, "", "").unwrap();
        assert!(output.contains("/home/user/photos/vacation.jpg"));
        assert!(output.contains("/backup/photos/vacation.jpg"));
    }

    #[test]
    fn test_html_renders_human_readable_file_sizes() {
        let dups = vec![make_group(&[
            ("/a/large.bin", 10 * 1024 * 1024),
            ("/b/large.bin", 10 * 1024 * 1024),
        ])];
        let output = html("Title", &dups, "", "").unwrap();
        assert!(output.contains("10") && (output.contains("MiB") || output.contains("MB")));
    }

    #[test]
    fn test_html_file_links_use_file_scheme() {
        let dups = vec![make_group(&[
            ("/some/path/file.txt", 100),
            ("/other/path/file.txt", 100),
        ])];
        let output = html("Title", &dups, "", "").unwrap();
        assert!(output.contains("href=\"file:///some/path/file.txt\""));
        assert!(output.contains("href=\"file:///other/path/file.txt\""));
    }

    #[test]
    fn test_html_has_dark_mode_media_query() {
        let output = html("Title", &Vec::new(), "", "").unwrap();
        assert!(output.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn test_html_file_size_has_user_select_none() {
        let output = html("Title", &Vec::new(), "", "").unwrap();
        assert!(output.contains("user-select: none"));
    }

    #[test]
    fn test_html_has_content_visibility_for_large_list_performance() {
        let output = html("Title", &Vec::new(), "", "").unwrap();
        assert!(output.contains("content-visibility: auto"));
    }

    #[test]
    fn test_html_header_is_sticky() {
        let output = html("Title", &Vec::new(), "", "").unwrap();
        assert!(output.contains("position: sticky"));
    }

    #[test]
    fn test_html_file_writes_output_with_timing() {
        let dups = vec![make_group(&[
            ("/a/duplicate.txt", 256),
            ("/b/duplicate.txt", 256),
        ])];
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("report.html");
        html_file(
            path.to_str().unwrap(),
            "Test Title",
            &dups,
            "2024-06-01 12:00:00 +0000",
            "5s",
        )
        .unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Test Title"));
        assert!(content.contains("/a/duplicate.txt"));
        assert!(content.contains("2024-06-01 12:00:00 +0000"));
        assert!(content.contains("5s"));
    }

    #[test]
    fn test_html_title_appears_in_page_title_and_heading() {
        let output = html("My Scan Path", &Vec::new(), "", "").unwrap();
        assert!(output.contains("<title>Duplicates of My Scan Path</title>"));
        assert!(output.contains("Duplicates of: My Scan Path"));
    }

    #[test]
    fn test_html_no_unrendered_template_variables() {
        let dups = vec![
            make_group(&[("/a/f1.txt", 100), ("/b/f1.txt", 100)]),
            make_group(&[("/c/f2.png", 500), ("/d/f2.png", 500)]),
        ];
        let output = html("Title", &dups, "2024-01-01 00:00:00 +0000", "30s").unwrap();
        assert!(!output.contains("{{"));
        assert!(!output.contains("}}"));
    }
}

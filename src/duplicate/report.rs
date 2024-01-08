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

use handlebars::Handlebars;
use serde_json::json;
use std::{fs, vec::Vec};

const DUPLICATE_REPORT_HTML: &str = include_str!("embed/templates/duplicate-report.html");

pub(crate) fn html_file(
    output_path: &str,
    title: &str,
    dups: &Vec<Vec<crate::common::fs::FileMetadata>>,
) -> std::io::Result<()> {
    let contents = html(title, dups).unwrap();
    fs::write(output_path, contents)
}

pub(crate) fn html(
    title: &str,
    dups: &Vec<Vec<crate::common::fs::FileMetadata>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut reg = Handlebars::new();
    reg.set_strict_mode(true);
    reg.register_helper("humansize", Box::new(humansize));
    reg.register_template_string("duplicate-report", DUPLICATE_REPORT_HTML)?;
    Ok(reg.render(
        "duplicate-report",
        &json!({
            "title": title,
            "groups": dups,
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
        write!(out, "{}", rendered)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty_html() {
        let dups = Vec::new();
        assert_eq!(
            include_str!("embed/testdata/test-report.html"),
            html("Duplicate Title String", &dups).unwrap()
        );
    }
}

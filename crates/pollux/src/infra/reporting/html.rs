// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use std::env::current_dir;
use std::fs;
use std::path::PathBuf;

static TEMPLATE: &str = include_str!("template.html");

pub struct HtmlReporter {
    output_folder: PathBuf,
}

impl Default for HtmlReporter {
    fn default() -> Self {
        Self::new(current_dir().unwrap())
    }
}

impl HtmlReporter {
    pub fn new(output_folder: PathBuf) -> Self {
        Self { output_folder }
    }

    pub fn report_ecosystem_inquired(&self) {
        let report_file = self.output_folder.join("pollux-report.html");
        fs::write(report_file, TEMPLATE).unwrap();
    }
}

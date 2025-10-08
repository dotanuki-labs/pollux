// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::EcosystemInquiringResults;
use minijinja::Environment;
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

    pub fn report_ecosystem_inquired(&self, results: &EcosystemInquiringResults) {
        let report_file = self.output_folder.join("pollux-report.html");
        let mut env = Environment::new();
        env.add_template("pollux-report", TEMPLATE)
            .expect("failed to add template");

        let template = env.get_template("pollux-report").unwrap();

        let rendered = template.render(results).expect("failed to render results");
        fs::write(report_file.clone(), rendered).unwrap();

        println!();
        println!("Report available at : {} ", report_file.to_str().unwrap());
        println!();
    }
}

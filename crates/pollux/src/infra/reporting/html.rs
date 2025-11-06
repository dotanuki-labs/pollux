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
        Self::new(current_dir().expect("failed to evaluate current directory"))
    }
}

impl HtmlReporter {
    pub fn new(output_folder: PathBuf) -> Self {
        Self { output_folder }
    }

    pub fn report_ecosystem_inquired(&self, results: &EcosystemInquiringResults) -> anyhow::Result<()> {
        let report_file = self.output_folder.join("pollux-report.html");
        let mut env = Environment::new();
        env.add_template("pollux-report", TEMPLATE)?;
        let template = env.get_template("pollux-report")?;

        let rendered = template.render(results)?;
        fs::write(report_file.clone(), rendered)?;

        println!();
        println!("Report available at : {:?} ", report_file);
        println!();

        Ok(())
    }
}

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod analyser;
pub mod checker;
pub mod cleaner;
pub mod inquirer;

use crate::core::models::{CargoPackage, CleanupScope, InquireReportKind};
use crate::infra::reporting::console::ConsoleReporter;
use crate::infra::reporting::html::HtmlReporter;
use crate::pollux::PolluxTask::*;
use crate::pollux::inquirer::PolluxInquirer;
use analyser::PolluxAnalyser;
use checker::PolluxChecker;
use cleaner::PolluxCleaner;
use std::path::PathBuf;

pub enum PolluxTask {
    AnalyseRustCrate(CargoPackage),
    AnalyseRustProject(PathBuf),
    CheckRustCrate(CargoPackage),
    CleanupAnalysedData,
    CleanupPackageSource,
    CleanupEverything,
    InquirePopularCrates(InquireReportKind),
}

pub struct Pollux {
    cleaner: PolluxCleaner,
    analyser: PolluxAnalyser,
    checker: PolluxChecker,
    inquirer: PolluxInquirer,
    console_reporter: ConsoleReporter,
    html_reporter: HtmlReporter,
}

impl Pollux {
    pub fn new(
        cleaner: PolluxCleaner,
        analyser: PolluxAnalyser,
        checker: PolluxChecker,
        inquirer: PolluxInquirer,
        console_reporter: ConsoleReporter,
        html_reporter: HtmlReporter,
    ) -> Self {
        Self {
            cleaner,
            analyser,
            checker,
            inquirer,
            console_reporter,
            html_reporter,
        }
    }

    pub async fn execute(self, task: PolluxTask) -> anyhow::Result<()> {
        match task {
            AnalyseRustProject(project_root) => self.analyse_rust_project(project_root).await?,
            AnalyseRustCrate(cargo_package) => self.analyse_cargo_package(&cargo_package).await?,
            CheckRustCrate(cargo_package) => self.check_individual_crate(&cargo_package).await?,
            CleanupEverything => self.cleanup_everything(),
            CleanupPackageSource => self.cleanup_packages(),
            CleanupAnalysedData => self.cleanup_analysed_data(),
            InquirePopularCrates(report_kind) => self.inquire_popular_crates(report_kind).await?,
        }

        Ok(())
    }

    async fn analyse_cargo_package(self, cargo_package: &CargoPackage) -> anyhow::Result<()> {
        self.console_reporter.report_pollux_started();
        let results = self.analyser.analyse_package(cargo_package).await?;
        self.console_reporter.report_analyser_outcomes(&results);
        Ok(())
    }

    async fn analyse_rust_project(self, project_root: PathBuf) -> anyhow::Result<()> {
        self.console_reporter.report_pollux_started();
        let results = self.analyser.analyse_project(project_root.as_path()).await?;
        self.console_reporter.report_analyser_outcomes(&results);
        Ok(())
    }

    async fn check_individual_crate(self, cargo_package: &CargoPackage) -> anyhow::Result<()> {
        self.console_reporter.report_checker_started(cargo_package);
        let check = self.checker.check_package(cargo_package).await?;
        self.console_reporter.report_checker_outcomes(check);
        Ok(())
    }

    fn cleanup_everything(self) {
        self.cleaner.cleanup_everything();
        self.console_reporter.report_cleaning_finished(CleanupScope::Everything)
    }

    fn cleanup_analysed_data(self) {
        self.cleaner.cleanup_analysed_data();
        self.console_reporter
            .report_cleaning_finished(CleanupScope::AnalysedData)
    }

    fn cleanup_packages(self) {
        self.cleaner.cleanup_package_sources();
        self.console_reporter
            .report_cleaning_finished(CleanupScope::PackageSources)
    }

    async fn inquire_popular_crates(&self, report_kind: InquireReportKind) -> anyhow::Result<()> {
        self.console_reporter.report_pollux_started();
        let outcomes = self.inquirer.inquire_most_popular_crates().await?;

        match report_kind {
            InquireReportKind::Console => self.console_reporter.report_ecosystem_inquired(&outcomes),
            InquireReportKind::Html => self.html_reporter.report_ecosystem_inquired(&outcomes)?,
        }

        Ok(())
    }
}

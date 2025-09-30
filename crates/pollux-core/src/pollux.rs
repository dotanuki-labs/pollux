// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub(crate) mod analyser;
pub(crate) mod checker;
pub(crate) mod cleaner;

use crate::domain::models::{CargoPackage, CleanupScope};
use crate::infra::cli::reporter::ConsoleReporter;
use crate::pollux::PolluxTask::{
    AnalyseRustCrate, AnalyseRustProject, CheckRustCrate, CleanupAnalysedData, CleanupEverything, CleanupPackageSource,
};
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
}

pub struct Pollux {
    cleaner: PolluxCleaner,
    analyser: PolluxAnalyser,
    checker: PolluxChecker,
    console_reporter: ConsoleReporter,
}

impl Pollux {
    pub(crate) fn new(
        cleaner: PolluxCleaner,
        analyser: PolluxAnalyser,
        checker: PolluxChecker,
        console_reporter: ConsoleReporter,
    ) -> Self {
        Self {
            cleaner,
            analyser,
            checker,
            console_reporter,
        }
    }

    pub async fn execute(self, task: PolluxTask) -> anyhow::Result<()> {
        match task {
            AnalyseRustProject(project_root) => {
                self.console_reporter.report_analyser_started();
                let results = self.analyser.analyse_project(project_root.as_path()).await?;
                self.console_reporter.report_analyser_outcomes(&results);
            },
            AnalyseRustCrate(cargo_package) => {
                self.console_reporter.report_analyser_started();
                let results = self.analyser.analyse_package(&cargo_package).await?;
                self.console_reporter.report_analyser_outcomes(&results);
            },
            CheckRustCrate(cargo_package) => {
                self.console_reporter.report_checker_started(&cargo_package);
                let check = self.checker.check_package(&cargo_package).await?;
                self.console_reporter.report_checker_outcomes(check);
            },
            CleanupEverything => {
                self.cleaner.cleanup_everything();
                self.console_reporter.report_cleaning_finished(CleanupScope::Everything)
            },
            CleanupPackageSource => {
                self.cleaner.cleanup_package_sources();
                self.console_reporter
                    .report_cleaning_finished(CleanupScope::PackageSources)
            },
            CleanupAnalysedData => {
                self.cleaner.cleanup_analysed_data();
                self.console_reporter
                    .report_cleaning_finished(CleanupScope::AnalysedData)
            },
        }

        Ok(())
    }
}

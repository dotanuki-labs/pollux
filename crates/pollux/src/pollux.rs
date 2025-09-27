// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod analyser;
pub mod checker;
pub mod cleaner;

use crate::core::models::CargoPackage;
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
}

impl Pollux {
    pub fn new(cleaner: PolluxCleaner, analyser: PolluxAnalyser, checker: PolluxChecker) -> Self {
        Self {
            cleaner,
            analyser,
            checker,
        }
    }

    pub async fn execute(self, task: PolluxTask) -> anyhow::Result<()> {
        match task {
            AnalyseRustProject(project_root) => self.analyser.analyse_project(project_root.as_path()).await,
            AnalyseRustCrate(cargo_package) => self.analyser.analyse_package(&cargo_package).await,
            CheckRustCrate(cargo_package) => self.checker.check_package(&cargo_package).await,
            CleanupEverything => self.cleaner.cleanup_everything(),
            CleanupPackageSource => self.cleaner.cleanup_package_sources(),
            CleanupAnalysedData => self.cleaner.cleanup_analysed_data(),
        }
    }
}

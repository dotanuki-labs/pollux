// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod checker;
pub mod cleaner;
pub mod evaluator;

use crate::core::models::CargoPackage;
use crate::pollux::PolluxTask::{
    CheckRustCrate, CleanupEvaluations, CleanupEverything, CleanupPackages, EvaluateRustCrate, EvaluateRustProject,
};
use checker::PolluxCrateChecker;
use cleaner::PolluxCleaner;
use evaluator::PolluxEvaluatorActor;
use std::path::PathBuf;

pub enum PolluxTask {
    CheckRustCrate(CargoPackage),
    CleanupEverything,
    CleanupPackages,
    CleanupEvaluations,
    EvaluateRustProject(PathBuf),
    EvaluateRustCrate(CargoPackage),
}

pub struct Pollux {
    cleaner: PolluxCleaner,
    evaluator: PolluxEvaluatorActor,
    checker: PolluxCrateChecker,
}

impl Pollux {
    pub fn new(cleaner: PolluxCleaner, evaluator: PolluxEvaluatorActor, checker: PolluxCrateChecker) -> Self {
        Self {
            cleaner,
            evaluator,
            checker,
        }
    }

    pub async fn execute(self, task: PolluxTask) -> anyhow::Result<()> {
        match task {
            EvaluateRustProject(project_root) => self.evaluator.evaluate_local_project(project_root.as_path()).await,
            EvaluateRustCrate(cargo_package) => self.evaluator.evaluate_crate_package(&cargo_package).await,
            CleanupEverything => self.cleaner.cleanup_everything(),
            CleanupPackages => self.cleaner.cleanup_cached_packages(),
            CleanupEvaluations => self.cleaner.cleanup_cached_evaluations(),
            CheckRustCrate(cargo_package) => self.checker.check(&cargo_package).await,
        }
    }
}

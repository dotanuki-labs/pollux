// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::domain::analysers::combined::VeracityChecksAnalyser;
use crate::domain::interfaces::CrateVeracityAnalysis;
use crate::domain::models::{CargoPackage, CrateVeracityChecks};

pub(crate) struct PolluxChecker {
    veracity_analyser: VeracityChecksAnalyser,
}

impl PolluxChecker {
    pub fn new(veracity_analyser: VeracityChecksAnalyser) -> Self {
        Self { veracity_analyser }
    }

    pub async fn check_package(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateVeracityChecks> {
        log::info!("[pollux.checker] starting evaluation for package {}", cargo_package);
        let check = self.veracity_analyser.execute(cargo_package).await;
        log::info!("[pollux.checker] finished evaluation for package {}", cargo_package);
        check
    }
}

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::{CargoPackage, CrateVeracityLevel};

pub trait VeracityFactorEvaluation {
    async fn evaluate(&self, cargo_package: &CargoPackage) -> anyhow::Result<bool>;
}

pub trait CrateVeracityLevelEvaluation {
    async fn evaluate(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateVeracityLevel>;
}

pub trait VeracityEvaluationStorage {
    fn retrieve_evaluation(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateVeracityLevel>;
    fn save_evaluation(&self, cargo_package: &CargoPackage, veracity_level: CrateVeracityLevel) -> anyhow::Result<()>;
}

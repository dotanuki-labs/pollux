// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::{CargoPackage, CrateVeracityLevel};

pub trait VeracityEvaluation {
    async fn evaluate(&self, crate_info: &CargoPackage) -> anyhow::Result<bool>;
}

pub trait CrateVeracityEvaluation {
    async fn evaluate(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityLevel>;
}

pub trait VeracityEvaluationStorage {
    fn read(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityLevel>;
    fn save(&self, crate_info: &CargoPackage, veracity_level: CrateVeracityLevel) -> anyhow::Result<()>;
}

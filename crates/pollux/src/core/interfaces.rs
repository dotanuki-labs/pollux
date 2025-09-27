// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::{CargoPackage, CrateVeracityLevel};

pub trait VeracityFactorCheck {
    async fn execute(&self, cargo_package: &CargoPackage) -> anyhow::Result<bool>;
}

pub trait CrateVeracityAnalysis {
    async fn execute(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateVeracityLevel>;
}

pub trait AnalyzedDataStorage {
    fn retrieve(&self, cargo_package: &CargoPackage) -> anyhow::Result<Option<CrateVeracityLevel>>;
    fn save(&self, cargo_package: &CargoPackage, veracity_level: CrateVeracityLevel) -> anyhow::Result<()>;
}

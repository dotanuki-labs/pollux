// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::{CargoPackage, CrateVeracityChecks};
use url::Url;

pub trait VeracityFactorCheck {
    async fn execute(&self, cargo_package: &CargoPackage) -> anyhow::Result<Option<Url>>;
}

pub trait CrateVeracityAnalysis {
    async fn execute(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateVeracityChecks>;
}

pub trait AnalyzedDataStorage {
    fn retrieve(&self, cargo_package: &CargoPackage) -> anyhow::Result<Option<CrateVeracityChecks>>;
    fn save(&self, cargo_package: &CargoPackage, veracity_level: CrateVeracityChecks) -> anyhow::Result<()>;
}

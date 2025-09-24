// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::{CargoPackage, CrateVeracityLevel};
use std::path::Path;

pub trait VeracityFactorEvaluation {
    async fn evaluate(&self, cargo_package: &CargoPackage) -> anyhow::Result<bool>;
}

pub trait CrateVeracityLevelEvaluation {
    async fn evaluate(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateVeracityLevel>;
}

pub trait VeracityEvaluationStorage {
    fn read(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateVeracityLevel>;
    fn save(&self, cargo_package: &CargoPackage, veracity_level: CrateVeracityLevel) -> anyhow::Result<()>;
}

pub trait PackagesResolution {
    async fn resolve_for_local_project(&self, project_path: &Path) -> anyhow::Result<Vec<CargoPackage>>;
    async fn resolve_for_crate_package(&self, cargo_package: &CargoPackage) -> anyhow::Result<Vec<CargoPackage>>;
}

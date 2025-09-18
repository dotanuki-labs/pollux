// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::infra::{CrateBuildReproducibilityEvaluator, CrateProvenanceEvaluator};
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
}

impl CrateInfo {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    #[cfg(test)]
    pub fn with(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}

impl Display for CrateInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("pkg:cargo/{}@{}", self.name, self.version))
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum VeracityFactor {
    ReproducibleBuilds,
    ProvenanceAttested,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum CrateVeracityLevel {
    NotAvailable,
    SingleFactor(VeracityFactor),
    TwoFactors,
}

#[allow(dead_code)]
pub trait VeracityEvaluation {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool>;
}

pub struct CrateVeracityEvaluator {
    provenance: CrateProvenanceEvaluator,
    reproducibility: CrateBuildReproducibilityEvaluator,
}

impl CrateVeracityEvaluator {
    pub async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<CrateVeracityLevel> {
        let uses_trusted_publishing = self.provenance.evaluate(crate_info).await?;
        let has_reproduced_build = self.reproducibility.evaluate(crate_info).await?;

        let verification = match (uses_trusted_publishing, has_reproduced_build) {
            (true, true) => CrateVeracityLevel::TwoFactors,
            (false, true) => CrateVeracityLevel::SingleFactor(VeracityFactor::ReproducibleBuilds),
            (true, false) => CrateVeracityLevel::SingleFactor(VeracityFactor::ProvenanceAttested),
            (false, false) => CrateVeracityLevel::NotAvailable,
        };

        Ok(verification)
    }

    fn new(provenance: CrateProvenanceEvaluator, reproducibility: CrateBuildReproducibilityEvaluator) -> Self {
        Self {
            provenance,
            reproducibility,
        }
    }
}

pub mod factory {
    use crate::core::CrateVeracityEvaluator;
    use crate::infra::{CrateBuildReproducibilityEvaluator, CrateProvenanceEvaluator};

    pub fn create_veracity_evaluator(
        provenance_factory: fn() -> CrateProvenanceEvaluator,
        reproducibility_factory: fn() -> CrateBuildReproducibilityEvaluator,
    ) -> CrateVeracityEvaluator {
        CrateVeracityEvaluator::new(provenance_factory(), reproducibility_factory())
    }
}

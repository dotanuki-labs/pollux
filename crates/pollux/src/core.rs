// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::infra::{
    CachedVeracityEvaluator, CrateBuildReproducibilityEvaluator, CrateProvenanceEvaluator, VeracityEvaluationStorage,
};
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

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum VeracityFactor {
    ReproducibleBuilds,
    ProvenanceAttested,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum CrateVeracityLevel {
    NotAvailable,
    SingleFactor(VeracityFactor),
    TwoFactors,
}

impl CrateVeracityLevel {
    pub fn to_booleans(&self) -> (bool, bool) {
        match self {
            CrateVeracityLevel::NotAvailable => (false, false),
            CrateVeracityLevel::SingleFactor(factor) => match factor {
                VeracityFactor::ReproducibleBuilds => (false, true),
                VeracityFactor::ProvenanceAttested => (true, false),
            },
            CrateVeracityLevel::TwoFactors => (true, true),
        }
    }

    pub fn from_booleans(provenance: bool, rebuilds: bool) -> Self {
        match (provenance, rebuilds) {
            (true, true) => CrateVeracityLevel::TwoFactors,
            (false, true) => CrateVeracityLevel::SingleFactor(VeracityFactor::ReproducibleBuilds),
            (true, false) => CrateVeracityLevel::SingleFactor(VeracityFactor::ProvenanceAttested),
            (false, false) => CrateVeracityLevel::NotAvailable,
        }
    }
}

pub trait VeracityEvaluation {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool>;
}

pub trait CrateVeracityEvaluation {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<CrateVeracityLevel>;
}

pub struct CombinedVeracityEvaluator {
    cache: CachedVeracityEvaluator,
    provenance: CrateProvenanceEvaluator,
    reproducibility: CrateBuildReproducibilityEvaluator,
}

impl CombinedVeracityEvaluator {
    fn new(
        cache: CachedVeracityEvaluator,
        provenance: CrateProvenanceEvaluator,
        reproducibility: CrateBuildReproducibilityEvaluator,
    ) -> Self {
        Self {
            cache,
            provenance,
            reproducibility,
        }
    }

    async fn evaluate_two_veracity_factors(&self, crate_info: &CrateInfo) -> anyhow::Result<CrateVeracityLevel> {
        let has_provenance = self.provenance.evaluate(crate_info).await?;
        let has_reproduced_build = self.reproducibility.evaluate(crate_info).await?;

        let veracity_level = CrateVeracityLevel::from_booleans(has_provenance, has_reproduced_build);
        self.cache.save(crate_info, veracity_level.clone())?;
        Ok(veracity_level)
    }

    async fn evaluate_missing_veracity_factor(
        &self,
        existing_factor: VeracityFactor,
        crate_info: &CrateInfo,
    ) -> anyhow::Result<CrateVeracityLevel> {
        let found_additional_factor = match existing_factor {
            VeracityFactor::ReproducibleBuilds => self.provenance.evaluate(crate_info).await?,
            VeracityFactor::ProvenanceAttested => self.reproducibility.evaluate(crate_info).await?,
        };

        let new_veracity_level = match found_additional_factor {
            true => {
                self.cache.save(crate_info, CrateVeracityLevel::TwoFactors)?;
                CrateVeracityLevel::TwoFactors
            },
            false => CrateVeracityLevel::SingleFactor(existing_factor),
        };

        Ok(new_veracity_level)
    }
}

impl CrateVeracityEvaluation for CombinedVeracityEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<CrateVeracityLevel> {
        let cached_veracity = self.cache.read(crate_info).unwrap_or(CrateVeracityLevel::NotAvailable);

        match cached_veracity {
            CrateVeracityLevel::NotAvailable => self.evaluate_two_veracity_factors(crate_info).await,
            CrateVeracityLevel::SingleFactor(factor) => self.evaluate_missing_veracity_factor(factor, crate_info).await,
            CrateVeracityLevel::TwoFactors => Ok(cached_veracity),
        }
    }
}

pub mod factory {
    use crate::core::CombinedVeracityEvaluator;
    use crate::infra::{CachedVeracityEvaluator, CrateBuildReproducibilityEvaluator, CrateProvenanceEvaluator};

    pub fn create_veracity_evaluator(
        cached_factory: fn() -> CachedVeracityEvaluator,
        provenance_factory: fn() -> CrateProvenanceEvaluator,
        reproducibility_factory: fn() -> CrateBuildReproducibilityEvaluator,
    ) -> CombinedVeracityEvaluator {
        CombinedVeracityEvaluator::new(cached_factory(), provenance_factory(), reproducibility_factory())
    }
}

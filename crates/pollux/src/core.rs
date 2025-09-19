// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::infra::{
    CachedVeracityEvaluator, CrateBuildReproducibilityEvaluator, CrateProvenanceEvaluator, VeracityEvaluationStorage,
};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use crate::core::{
        CombinedVeracityEvaluator, CrateInfo, CrateVeracityEvaluation, CrateVeracityLevel, VeracityFactor,
    };
    use crate::infra::{
        CachedVeracityEvaluator, CrateBuildReproducibilityEvaluator, CrateProvenanceEvaluator, FakeVeracityEvaluator,
    };
    use assertor::{BooleanAssertion, EqualityAssertion};
    use std::collections::HashMap;

    struct CrateScenario {
        name: &'static str,
        version: &'static str,
        provenance: bool,
        reproducible: bool,
        cached_results: bool,
        expected: CrateVeracityLevel,
    }

    fn fake_results_storage(scenario: &CrateScenario) -> HashMap<String, CrateVeracityLevel> {
        if scenario.cached_results {
            let cache_key = CrateInfo::with(scenario.name, scenario.version).to_string();
            let veracity_level = CrateVeracityLevel::from_booleans(scenario.provenance, scenario.reproducible);
            HashMap::from([(cache_key, veracity_level)])
        } else {
            HashMap::new()
        }
    }

    fn fake_provenance_evaluator(scenario: &CrateScenario) -> FakeVeracityEvaluator {
        if scenario.provenance {
            let info = CrateInfo::with(scenario.name, scenario.version);
            FakeVeracityEvaluator(vec![info])
        } else {
            FakeVeracityEvaluator(vec![])
        }
    }

    fn fake_reproducibility_evaluator(scenario: &CrateScenario) -> FakeVeracityEvaluator {
        if scenario.reproducible {
            let info = CrateInfo::with(scenario.name, scenario.version);
            FakeVeracityEvaluator(vec![info])
        } else {
            FakeVeracityEvaluator(vec![])
        }
    }

    #[tokio::test]
    async fn test_veracity_evaluator_scenarios() {
        let scenarios = vec![
            CrateScenario {
                name: "canopus",
                version: "0.1.1",
                provenance: false,
                reproducible: false,
                cached_results: false,
                expected: CrateVeracityLevel::NotAvailable,
            },
            CrateScenario {
                name: "castaway",
                version: "0.2.2",
                provenance: false,
                reproducible: true,
                cached_results: false,
                expected: CrateVeracityLevel::SingleFactor(VeracityFactor::ReproducibleBuilds),
            },
            CrateScenario {
                name: "castaway",
                version: "0.2.4",
                provenance: false,
                reproducible: false,
                cached_results: true,
                expected: CrateVeracityLevel::NotAvailable,
            },
            CrateScenario {
                name: "bon",
                version: "3.7.2",
                provenance: true,
                reproducible: false,
                cached_results: true,
                expected: CrateVeracityLevel::SingleFactor(VeracityFactor::ProvenanceAttested),
            },
        ];

        for scenario in scenarios {
            let crate_info = CrateInfo::with(scenario.name, scenario.version);
            let previous_veracity_level = CrateVeracityLevel::from_booleans(scenario.provenance, scenario.reproducible);

            let veracity_evaluator = CombinedVeracityEvaluator::new(
                CachedVeracityEvaluator::FakeCache(fake_results_storage(&scenario)),
                CrateProvenanceEvaluator::FakeRegistry(fake_provenance_evaluator(&scenario)),
                CrateBuildReproducibilityEvaluator::FakeRebuilder(fake_reproducibility_evaluator(&scenario)),
            );

            let evaluation = veracity_evaluator.evaluate(&crate_info).await.unwrap();

            assertor::assert_that!(evaluation).is_equal_to(scenario.expected);

            if previous_veracity_level == CrateVeracityLevel::NotAvailable
                && evaluation != CrateVeracityLevel::NotAvailable
            {
                match veracity_evaluator.cache {
                    CachedVeracityEvaluator::FakeCache(entries) => {
                        assertor::assert_that!(entries.contains_key(scenario.name)).is_true()
                    },
                    _ => panic!("Not allowed on this test"),
                }
            }
        }
    }
}

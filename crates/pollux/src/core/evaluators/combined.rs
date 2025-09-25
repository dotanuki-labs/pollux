// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::evaluators::standalone::{
    BuildReproducibilityEvaluator, CachedExecutionEvaluator, CrateProvenanceEvaluator,
};
use crate::core::interfaces::{CrateVeracityLevelEvaluation, VeracityEvaluationStorage, VeracityFactorEvaluation};
use crate::core::models::{CargoPackage, CrateVeracityLevel, VeracityFactor};

pub struct CombinedVeracityEvaluator {
    cache: CachedExecutionEvaluator,
    provenance: CrateProvenanceEvaluator,
    reproducibility: BuildReproducibilityEvaluator,
}

impl CombinedVeracityEvaluator {
    pub fn new(
        cache: CachedExecutionEvaluator,
        provenance: CrateProvenanceEvaluator,
        reproducibility: BuildReproducibilityEvaluator,
    ) -> Self {
        Self {
            cache,
            provenance,
            reproducibility,
        }
    }

    async fn evaluate_two_veracity_factors(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityLevel> {
        let has_reproduced_build = self.reproducibility.evaluate(crate_info).await?;
        let has_provenance = self.provenance.evaluate(crate_info).await?;

        let veracity_level = CrateVeracityLevel::from_booleans(has_provenance, has_reproduced_build);
        self.cache.save_evaluation(crate_info, veracity_level.clone())?;
        Ok(veracity_level)
    }

    async fn re_evaluate_reproducibility(
        &self,
        has_provenance: bool,
        cargo_package: &CargoPackage,
    ) -> anyhow::Result<CrateVeracityLevel> {
        log::info!("[pollux.core] will re-evaluate reproducibility for {}", cargo_package);
        let is_reproducible = self.reproducibility.evaluate(cargo_package).await?;
        let new_veracity_level = CrateVeracityLevel::from_booleans(has_provenance, is_reproducible);
        self.cache.save_evaluation(cargo_package, new_veracity_level.clone())?;
        Ok(new_veracity_level)
    }
}

impl CrateVeracityLevelEvaluation for CombinedVeracityEvaluator {
    async fn evaluate(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateVeracityLevel> {
        let Some(cached_veracity_level) = self.cache.retrieve_evaluation(cargo_package)? else {
            return self.evaluate_two_veracity_factors(cargo_package).await;
        };

        let new_evaluation = match &cached_veracity_level {
            CrateVeracityLevel::NotAvailable => self.re_evaluate_reproducibility(false, cargo_package).await,
            CrateVeracityLevel::SingleFactor(VeracityFactor::ProvenanceAttested) => {
                self.re_evaluate_reproducibility(true, cargo_package).await
            },
            _ => Ok(cached_veracity_level.clone()),
        };

        if new_evaluation.is_ok() {
            return new_evaluation;
        }

        log::info!(
            "[pollux.core] failed to evaluate {} | reason = {}; defaulting to cache",
            cargo_package,
            new_evaluation.unwrap_err()
        );
        log::info!("[pollux.core] using cached veracity evaluation for {}", cargo_package);
        Ok(cached_veracity_level)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::evaluators::combined::CombinedVeracityEvaluator;
    use crate::core::evaluators::standalone::{
        BuildReproducibilityEvaluator, CachedExecutionEvaluator, CrateProvenanceEvaluator, FakeVeracityEvaluator,
    };
    use crate::core::interfaces::CrateVeracityLevelEvaluation;
    use crate::core::models::{CargoPackage, CrateVeracityLevel, VeracityFactor};
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
            let cache_key = CargoPackage::with(scenario.name, scenario.version).to_string();
            let veracity_level = CrateVeracityLevel::from_booleans(scenario.provenance, scenario.reproducible);
            HashMap::from([(cache_key, veracity_level)])
        } else {
            HashMap::new()
        }
    }

    fn fake_provenance_evaluator(scenario: &CrateScenario) -> FakeVeracityEvaluator {
        if scenario.provenance {
            let info = CargoPackage::with(scenario.name, scenario.version);
            FakeVeracityEvaluator(vec![info])
        } else {
            FakeVeracityEvaluator(vec![])
        }
    }

    fn fake_reproducibility_evaluator(scenario: &CrateScenario) -> FakeVeracityEvaluator {
        if scenario.reproducible {
            let info = CargoPackage::with(scenario.name, scenario.version);
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
            let crate_info = CargoPackage::with(scenario.name, scenario.version);
            let previous_veracity_level = CrateVeracityLevel::from_booleans(scenario.provenance, scenario.reproducible);

            let veracity_evaluator = CombinedVeracityEvaluator::new(
                CachedExecutionEvaluator::FakeCache(fake_results_storage(&scenario)),
                CrateProvenanceEvaluator::FakeRegistry(fake_provenance_evaluator(&scenario)),
                BuildReproducibilityEvaluator::FakeRebuilder(fake_reproducibility_evaluator(&scenario)),
            );

            let evaluation = veracity_evaluator.evaluate(&crate_info).await.unwrap();

            assertor::assert_that!(evaluation).is_equal_to(scenario.expected);

            if previous_veracity_level == CrateVeracityLevel::NotAvailable
                && evaluation != CrateVeracityLevel::NotAvailable
            {
                match veracity_evaluator.cache {
                    CachedExecutionEvaluator::FakeCache(entries) => {
                        assertor::assert_that!(entries.contains_key(scenario.name)).is_true()
                    },
                    _ => panic!("Not allowed on this test"),
                }
            }
        }
    }
}

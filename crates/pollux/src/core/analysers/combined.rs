// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::analysers::standalone::{BuildReproducibilityChecker, CachedDataChecker, CrateProvenanceChecker};
use crate::core::interfaces::{AnalyzedDataStorage, CrateVeracityAnalysis, VeracityFactorCheck};
use crate::core::models::{CargoPackage, CrateVeracityLevel, VeracityFactor};

pub struct VeracityFactorsAnalyser {
    cache: CachedDataChecker,
    provenance: CrateProvenanceChecker,
    reproducibility: BuildReproducibilityChecker,
}

impl VeracityFactorsAnalyser {
    pub fn new(
        cache: CachedDataChecker,
        provenance: CrateProvenanceChecker,
        reproducibility: BuildReproducibilityChecker,
    ) -> Self {
        Self {
            cache,
            provenance,
            reproducibility,
        }
    }

    async fn analyse(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityLevel> {
        let has_reproduced_build = self.reproducibility.execute(crate_info).await?;
        let has_provenance = self.provenance.execute(crate_info).await?;

        let veracity_level = CrateVeracityLevel::from_booleans(has_provenance, has_reproduced_build);
        self.cache.save(crate_info, veracity_level.clone())?;
        Ok(veracity_level)
    }

    async fn re_check_reproducibility(
        &self,
        has_provenance: bool,
        cargo_package: &CargoPackage,
    ) -> anyhow::Result<CrateVeracityLevel> {
        log::info!("[pollux.core] will re-check reproducibility for {}", cargo_package);
        let is_reproducible = self.reproducibility.execute(cargo_package).await?;
        let new_veracity_level = CrateVeracityLevel::from_booleans(has_provenance, is_reproducible);
        self.cache.save(cargo_package, new_veracity_level.clone())?;
        Ok(new_veracity_level)
    }
}

impl CrateVeracityAnalysis for VeracityFactorsAnalyser {
    async fn execute(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateVeracityLevel> {
        let Some(cached_veracity_level) = self.cache.retrieve(cargo_package)? else {
            return self.analyse(cargo_package).await;
        };

        let new_analysis = match &cached_veracity_level {
            CrateVeracityLevel::NotAvailable => self.re_check_reproducibility(false, cargo_package).await,
            CrateVeracityLevel::SingleFactor(VeracityFactor::ProvenanceAttested) => {
                self.re_check_reproducibility(true, cargo_package).await
            },
            _ => Ok(cached_veracity_level.clone()),
        };

        if new_analysis.is_ok() {
            return new_analysis;
        }

        log::info!(
            "[pollux.core] failed to analyse {} | reason = {}; defaulting to cache",
            cargo_package,
            new_analysis.unwrap_err()
        );
        log::info!("[pollux.core] using cached veracity analysis for {}", cargo_package);
        Ok(cached_veracity_level)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::analysers::combined::VeracityFactorsAnalyser;
    use crate::core::analysers::standalone::{
        BuildReproducibilityChecker, CachedDataChecker, CrateProvenanceChecker, FakeVeracityChecker,
    };
    use crate::core::interfaces::CrateVeracityAnalysis;
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

    fn fake_provenance_checker(scenario: &CrateScenario) -> FakeVeracityChecker {
        if scenario.provenance {
            let info = CargoPackage::with(scenario.name, scenario.version);
            FakeVeracityChecker(vec![info])
        } else {
            FakeVeracityChecker(vec![])
        }
    }

    fn fake_reproducibility_checker(scenario: &CrateScenario) -> FakeVeracityChecker {
        if scenario.reproducible {
            let info = CargoPackage::with(scenario.name, scenario.version);
            FakeVeracityChecker(vec![info])
        } else {
            FakeVeracityChecker(vec![])
        }
    }

    #[tokio::test]
    async fn should_analyse_veracity_of_packages() {
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

            let veracity_analyser = VeracityFactorsAnalyser::new(
                CachedDataChecker::FakeCache(fake_results_storage(&scenario)),
                CrateProvenanceChecker::FakeRegistry(fake_provenance_checker(&scenario)),
                BuildReproducibilityChecker::FakeRebuilder(fake_reproducibility_checker(&scenario)),
            );

            let analysis = veracity_analyser.execute(&crate_info).await.unwrap();

            assertor::assert_that!(analysis).is_equal_to(scenario.expected);

            if previous_veracity_level == CrateVeracityLevel::NotAvailable
                && analysis != CrateVeracityLevel::NotAvailable
            {
                match veracity_analyser.cache {
                    CachedDataChecker::FakeCache(entries) => {
                        assertor::assert_that!(entries.contains_key(scenario.name)).is_true()
                    },
                    _ => panic!("Not allowed on this test"),
                }
            }
        }
    }
}

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::domain::analysers::standalone::{BuildReproducibilityChecker, CachedDataChecker, CrateProvenanceChecker};
use crate::domain::interfaces::{AnalyzedDataStorage, CrateVeracityAnalysis, VeracityFactorCheck};
use crate::domain::models::{CargoPackage, CrateVeracityChecks};

pub struct VeracityChecksAnalyser {
    cache: CachedDataChecker,
    provenance: CrateProvenanceChecker,
    reproducibility: BuildReproducibilityChecker,
}

impl VeracityChecksAnalyser {
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

    async fn analyse(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityChecks> {
        let reproducible_build_evidence = self.reproducibility.execute(crate_info).await?;
        let provenance_evidence = self.provenance.execute(crate_info).await?;

        let checks = CrateVeracityChecks::new(provenance_evidence, reproducible_build_evidence);
        self.cache.save(crate_info, checks.clone())?;
        Ok(checks)
    }
}

impl CrateVeracityAnalysis for VeracityChecksAnalyser {
    async fn execute(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateVeracityChecks> {
        let Some(cached_checks) = self.cache.retrieve(cargo_package)? else {
            return self.analyse(cargo_package).await;
        };

        let updated_reproducibility = match &cached_checks.reproducibility_evidence {
            None => {
                log::info!(
                    "[pollux.core] will re-check reproducibility for package {}",
                    cargo_package
                );
                self.reproducibility.execute(cargo_package).await?
            },
            Some(existing) => Some(existing.clone()),
        };

        let updated_checks = CrateVeracityChecks::new(cached_checks.provenance_evidence, updated_reproducibility);

        Ok(updated_checks)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::analysers::combined::VeracityChecksAnalyser;
    use crate::domain::analysers::standalone::{
        BuildReproducibilityChecker, CachedDataChecker, CrateProvenanceChecker, FakeVeracityChecker,
    };
    use crate::domain::interfaces::CrateVeracityAnalysis;
    use crate::domain::models::{CargoPackage, CrateVeracityChecks};
    use assertor::EqualityAssertion;
    use std::collections::HashMap;
    use std::str::FromStr;
    use url::Url;

    struct CrateScenario {
        name: &'static str,
        version: &'static str,
        provenance_evidence: Option<&'static str>,
        reproducibility_evidence: Option<&'static str>,
        serving_from_cache: bool,
    }

    fn convert_to_url(evidence: Option<&'static str>) -> Option<Url> {
        match evidence {
            None => None,
            Some(link) => Url::parse(link).ok(),
        }
    }

    fn fake_results_storage(scenario: &CrateScenario) -> HashMap<String, CrateVeracityChecks> {
        if scenario.serving_from_cache {
            let cache_key = CargoPackage::with(scenario.name, scenario.version).to_string();
            let checks = CrateVeracityChecks::new(
                convert_to_url(scenario.provenance_evidence),
                convert_to_url(scenario.reproducibility_evidence),
            );
            HashMap::from([(cache_key, checks)])
        } else {
            HashMap::new()
        }
    }

    fn fake_provenance_checker(scenario: &CrateScenario) -> FakeVeracityChecker {
        if scenario.provenance_evidence.is_some() {
            let package = CargoPackage::with(scenario.name, scenario.version);

            FakeVeracityChecker(HashMap::from([(
                package,
                scenario.provenance_evidence.unwrap().to_string(),
            )]))
        } else {
            FakeVeracityChecker(HashMap::new())
        }
    }

    fn fake_reproducibility_checker(scenario: &CrateScenario) -> FakeVeracityChecker {
        if scenario.reproducibility_evidence.is_some() {
            let package = CargoPackage::with(scenario.name, scenario.version);
            FakeVeracityChecker(HashMap::from([(
                package,
                scenario.reproducibility_evidence.unwrap().to_string(),
            )]))
        } else {
            FakeVeracityChecker(HashMap::new())
        }
    }

    fn crate_analyser(scenario: &CrateScenario) -> VeracityChecksAnalyser {
        VeracityChecksAnalyser::new(
            CachedDataChecker::FakeCache(fake_results_storage(scenario)),
            CrateProvenanceChecker::FakeRegistry(fake_provenance_checker(scenario)),
            BuildReproducibilityChecker::FakeRebuilder(fake_reproducibility_checker(scenario)),
        )
    }

    #[tokio::test]
    async fn should_run_online_analysis_and_get_two_veracity_checks() {
        let crate_name = "fake-crate";
        let crate_version = "1.2.3";
        let provenance_url = "https://shortn.ed/actions/runs/12345789";
        let ossrebuild_url = "https://shortn.ed/fake-crate-1.2.3.crate/rebuild.intoto.jsonl";

        let scenario = CrateScenario {
            name: crate_name,
            version: crate_version,
            provenance_evidence: Some(provenance_url),
            reproducibility_evidence: Some(ossrebuild_url),
            serving_from_cache: false,
        };

        let cargo_package = CargoPackage::with(scenario.name, scenario.version);

        let analyser = crate_analyser(&scenario);

        let analysed = analyser.execute(&cargo_package).await.unwrap();

        let expected = CrateVeracityChecks::new(Url::from_str(provenance_url).ok(), Url::from_str(ossrebuild_url).ok());

        assertor::assert_that!(analysed).is_equal_to(expected);
    }

    #[tokio::test]
    async fn should_run_online_analysis_and_get_one_veracity_check() {
        let crate_name = "fake-crate";
        let crate_version = "1.2.3";
        let ossrebuild_url = "https://shortn.ed/fake-crate-1.2.3.crate/rebuild.intoto.jsonl";

        let scenario = CrateScenario {
            name: crate_name,
            version: crate_version,
            provenance_evidence: None,
            reproducibility_evidence: Some(ossrebuild_url),
            serving_from_cache: false,
        };

        let cargo_package = CargoPackage::with(scenario.name, scenario.version);

        let analyser = crate_analyser(&scenario);

        let analysed = analyser.execute(&cargo_package).await.unwrap();

        let expected = CrateVeracityChecks::new(None, Url::from_str(ossrebuild_url).ok());

        assertor::assert_that!(analysed).is_equal_to(expected);
    }

    #[tokio::test]
    async fn should_run_get_veracity_checks_from_cache() {
        let crate_name = "fake-crate";
        let crate_version = "1.2.3";
        let provenance_url = "https://shortn.ed/actions/runs/12345789";

        let scenario = CrateScenario {
            name: crate_name,
            version: crate_version,
            provenance_evidence: Some(provenance_url),
            reproducibility_evidence: None,
            serving_from_cache: true,
        };

        let cargo_package = CargoPackage::with(scenario.name, scenario.version);

        let analyser = crate_analyser(&scenario);

        let analysed = analyser.execute(&cargo_package).await.unwrap();

        let expected = CrateVeracityChecks::new(Url::from_str(provenance_url).ok(), None);

        assertor::assert_that!(analysed).is_equal_to(expected);
    }
}

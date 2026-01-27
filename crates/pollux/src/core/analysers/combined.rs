// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::analysers::standalone::{
    BuildReproducibilityChecker, CachedDataChecker, CrateTrustedPublishingChecker,
};
use crate::core::interfaces::{AnalyzedDataStorage, CrateVeracityAnalysis, VeracityFactorCheck};
use crate::core::models::{CargoPackage, CrateVeracityChecks};

pub struct VeracityChecksAnalyser {
    cache: CachedDataChecker,
    trusted_publishing: CrateTrustedPublishingChecker,
    reproducibility: BuildReproducibilityChecker,
}

impl VeracityChecksAnalyser {
    pub fn new(
        cache: CachedDataChecker,
        trusted_publishing: CrateTrustedPublishingChecker,
        reproducibility: BuildReproducibilityChecker,
    ) -> Self {
        Self {
            cache,
            trusted_publishing,
            reproducibility,
        }
    }

    async fn analyse(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityChecks> {
        let reproducible_build_evidence = self.reproducibility.execute(crate_info).await?;
        let trusted_publishing_evidence = self.trusted_publishing.execute(crate_info).await?;

        let checks = CrateVeracityChecks::new(trusted_publishing_evidence, reproducible_build_evidence);
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

        let updated_checks =
            CrateVeracityChecks::new(cached_checks.trusted_publishing_evidence, updated_reproducibility);

        Ok(updated_checks)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::analysers::combined::VeracityChecksAnalyser;
    use crate::core::analysers::standalone::{
        BuildReproducibilityChecker, CachedDataChecker, CrateTrustedPublishingChecker, FakeVeracityChecker,
    };
    use crate::core::interfaces::CrateVeracityAnalysis;
    use crate::core::models::{CargoPackage, CrateVeracityChecks};
    use assertor::EqualityAssertion;
    use std::collections::HashMap;
    use std::str::FromStr;
    use url::Url;

    struct CrateScenario {
        name: &'static str,
        version: &'static str,
        trusted_publishing_evidence: Option<&'static str>,
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
                convert_to_url(scenario.trusted_publishing_evidence),
                convert_to_url(scenario.reproducibility_evidence),
            );
            HashMap::from([(cache_key, checks)])
        } else {
            HashMap::new()
        }
    }

    fn fake_trusted_publishing_checker(scenario: &CrateScenario) -> FakeVeracityChecker {
        let Some(evidence) = scenario.trusted_publishing_evidence else {
            return FakeVeracityChecker(HashMap::new());
        };

        let package = CargoPackage::with(scenario.name, scenario.version);

        FakeVeracityChecker(HashMap::from([(package, evidence.to_string())]))
    }

    fn fake_reproducibility_checker(scenario: &CrateScenario) -> FakeVeracityChecker {
        let Some(evidence) = scenario.reproducibility_evidence else {
            return FakeVeracityChecker(HashMap::new());
        };

        let package = CargoPackage::with(scenario.name, scenario.version);

        FakeVeracityChecker(HashMap::from([(package, evidence.to_string())]))
    }

    fn crate_analyser(scenario: &CrateScenario) -> VeracityChecksAnalyser {
        VeracityChecksAnalyser::new(
            CachedDataChecker::FakeCache(fake_results_storage(scenario)),
            CrateTrustedPublishingChecker::FakeRegistry(fake_trusted_publishing_checker(scenario)),
            BuildReproducibilityChecker::FakeRebuilder(fake_reproducibility_checker(scenario)),
        )
    }

    #[tokio::test]
    async fn should_run_online_analysis_and_get_two_veracity_checks() {
        let crate_name = "fake-crate";
        let crate_version = "1.2.3";
        let gha_run_url = "https://shortn.ed/actions/runs/12345789";
        let ossrebuild_url = "https://shortn.ed/fake-crate-1.2.3.crate/rebuild.intoto.jsonl";

        let scenario = CrateScenario {
            name: crate_name,
            version: crate_version,
            trusted_publishing_evidence: Some(gha_run_url),
            reproducibility_evidence: Some(ossrebuild_url),
            serving_from_cache: false,
        };

        let cargo_package = CargoPackage::with(scenario.name, scenario.version);

        let analyser = crate_analyser(&scenario);

        let analysed = analyser.execute(&cargo_package).await.unwrap();

        let expected = CrateVeracityChecks::new(Url::from_str(gha_run_url).ok(), Url::from_str(ossrebuild_url).ok());

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
            trusted_publishing_evidence: None,
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
        let gha_run_url = "https://shortn.ed/actions/runs/12345789";

        let scenario = CrateScenario {
            name: crate_name,
            version: crate_version,
            trusted_publishing_evidence: Some(gha_run_url),
            reproducibility_evidence: None,
            serving_from_cache: true,
        };

        let cargo_package = CargoPackage::with(scenario.name, scenario.version);

        let analyser = crate_analyser(&scenario);

        let analysed = analyser.execute(&cargo_package).await.unwrap();

        let expected = CrateVeracityChecks::new(Url::from_str(gha_run_url).ok(), None);

        assertor::assert_that!(analysed).is_equal_to(expected);
    }
}

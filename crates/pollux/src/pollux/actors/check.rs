// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::evaluators::combined::CombinedVeracityEvaluator;
use crate::core::interfaces::CrateVeracityLevelEvaluation;
use crate::core::models::CargoPackage;
use crate::infra::networking::ossrebuild::URL_OSS_REBUILD_CRATES;
use crate::pollux::actors::CrateChecks;
use std::str::FromStr;
use url::Url;

pub struct PolluxStandalonePackageChecker {
    veracity_evaluator: CombinedVeracityEvaluator,
}

impl PolluxStandalonePackageChecker {
    pub fn new(veracity_evaluator: CombinedVeracityEvaluator) -> Self {
        Self { veracity_evaluator }
    }

    pub async fn check(&self, cargo_package: &CargoPackage) -> anyhow::Result<CrateChecks> {
        log::info!("[pollux.checker] starting evaluation for package {}", cargo_package);
        let maybe_evaluated = self.veracity_evaluator.evaluate(cargo_package).await.ok();
        log::info!("[pollux.checker] finished evaluation for package {}", cargo_package);

        let Some(veracity_level) = maybe_evaluated else {
            return Ok(CrateChecks::new(None, None));
        };

        let (attested, reproducible) = veracity_level.to_booleans();

        let provenance_evidence = if attested {
            let crates_link = format!("https://crates.io/crates/{}/versions", cargo_package.name);
            Url::from_str(&crates_link).ok()
        } else {
            None
        };

        let reproducibility_evidence = if reproducible {
            let oss_rebuild_link = format!(
                "{}/{}/{}/{}-{}.crate/rebuild.intoto.jsonl",
                URL_OSS_REBUILD_CRATES,
                cargo_package.name,
                cargo_package.version,
                cargo_package.name,
                cargo_package.version
            );
            Url::from_str(&oss_rebuild_link).ok()
        } else {
            None
        };

        Ok(CrateChecks::new(provenance_evidence, reproducibility_evidence))
    }
}

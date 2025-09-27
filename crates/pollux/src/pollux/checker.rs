// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::evaluators::combined::CombinedVeracityEvaluator;
use crate::core::interfaces::CrateVeracityLevelEvaluation;
use crate::core::models::CargoPackage;
use crate::infra::networking::ossrebuild::URL_OSS_REBUILD_CRATES;
use console::style;
use std::str::FromStr;
use url::Url;

pub struct PolluxCrateChecker {
    veracity_evaluator: CombinedVeracityEvaluator,
}

impl PolluxCrateChecker {
    pub fn new(veracity_evaluator: CombinedVeracityEvaluator) -> Self {
        Self { veracity_evaluator }
    }

    pub async fn check(&self, cargo_package: &CargoPackage) -> anyhow::Result<()> {
        println!();
        println!("Checking veracity factors for : {}", cargo_package);
        println!();

        log::info!("[pollux.checker] starting evaluation for package {}", cargo_package);
        let maybe_evaluated = self.veracity_evaluator.evaluate(cargo_package).await.ok();
        log::info!("[pollux.checker] finished evaluation for package {}", cargo_package);

        let Some(veracity_level) = maybe_evaluated else {
            self.report_evidence(cargo_package, None, None);
            return Ok(());
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

        self.report_evidence(cargo_package, provenance_evidence, reproducibility_evidence);
        Ok(())
    }

    fn report_evidence(
        &self,
        cargo_package: &CargoPackage,
        provenance_evidence: Option<Url>,
        reproducibility_evidence: Option<Url>,
    ) {
        if let Some(cratesio_link) = provenance_evidence {
            println!(
                "• provenance evidence (v{} via github): {}",
                cargo_package.version,
                style(cratesio_link).cyan()
            );
        } else {
            println!("• provenance evidence : not found");
        }

        if let Some(oss_rebuild_link) = reproducibility_evidence {
            println!("• reproducibility evidence : {}", style(oss_rebuild_link).cyan());
        } else {
            println!("• reproducibility evidence : not found");
        }

        println!();
    }
}

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::analysers::combined::VeracityChecksAnalyser;
use crate::core::interfaces::CrateVeracityAnalysis;
use crate::core::models::CargoPackage;
use url::Url;

pub struct PolluxChecker {
    veracity_analyser: VeracityChecksAnalyser,
}

impl PolluxChecker {
    pub fn new(veracity_analyser: VeracityChecksAnalyser) -> Self {
        Self { veracity_analyser }
    }

    pub async fn check_package(&self, cargo_package: &CargoPackage) -> anyhow::Result<()> {
        println!();
        println!("Checking veracity factors for : {}", cargo_package);
        println!();

        log::info!("[pollux.checker] starting evaluation for package {}", cargo_package);
        let maybe_checked = self.veracity_analyser.execute(cargo_package).await.ok();
        log::info!("[pollux.checker] finished evaluation for package {}", cargo_package);

        let Some(veracity_checks) = maybe_checked else {
            self.report_evidence(None, None);
            return Ok(());
        };

        self.report_evidence(
            veracity_checks.provenance_evidence,
            veracity_checks.reproducibility_evidence,
        );
        Ok(())
    }

    fn report_evidence(&self, provenance_evidence: Option<Url>, reproducibility_evidence: Option<Url>) {
        if let Some(cratesio_link) = provenance_evidence {
            println!("• provenance evidence : {}", cratesio_link);
        } else {
            println!("• provenance evidence : not found");
        }

        if let Some(oss_rebuild_link) = reproducibility_evidence {
            println!("• reproducibility evidence : {}", oss_rebuild_link);
        } else {
            println!("• reproducibility evidence : not found");
        }

        println!();
    }
}

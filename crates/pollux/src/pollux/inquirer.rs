// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::analysers::combined::VeracityChecksAnalyser;
use crate::core::interfaces::CrateVeracityAnalysis;
use crate::core::models::{EcosystemInquiringResults, InquireCoverage, InquiringOutcome};
use crate::infra::networking::crates::PopularCratesFetcher;

pub struct PolluxInquirer {
    popular_crates_fetcher: PopularCratesFetcher,
    veracity_analyser: VeracityChecksAnalyser,
}

impl PolluxInquirer {
    pub fn new(popular_crates_fetcher: PopularCratesFetcher, veracity_analyser: VeracityChecksAnalyser) -> Self {
        Self {
            popular_crates_fetcher,
            veracity_analyser,
        }
    }

    pub async fn inquire_most_popular_crates(
        &self,
        coverage: InquireCoverage,
    ) -> anyhow::Result<EcosystemInquiringResults> {
        let popular_packages = self.popular_crates_fetcher.get_most_popular_crates(coverage).await?;

        let mut inquired_packages = vec![];
        let mut with_trusted_publishing = 0;
        let mut with_reproducibility = 0;

        for cargo_package in popular_packages {
            let checks = self.veracity_analyser.execute(&cargo_package).await?;

            match (&checks.trusted_publishing_evidence, &checks.reproducibility_evidence) {
                (Some(_), Some(_)) => {
                    with_trusted_publishing += 1;
                    with_reproducibility += 1;
                },
                (Some(_), None) => {
                    with_trusted_publishing += 1;
                },
                (None, Some(_)) => {
                    with_reproducibility += 1;
                },

                (None, None) => {
                    log::info!("[pollux.inquirer] not counting crate {}", cargo_package);
                },
            }

            inquired_packages.push(InquiringOutcome { cargo_package, checks });
        }

        let total_packages = inquired_packages.len() as u32;

        let results = EcosystemInquiringResults {
            total_crates_inquired: total_packages,
            total_crates_with_trusted_publishing: with_trusted_publishing,
            total_crates_with_reproducibility: with_reproducibility,
            presence_of_trusted_publishing: format!("{}", 100 * with_trusted_publishing / total_packages),
            presence_of_reproducibility: format!("{}", 100 * with_reproducibility / total_packages),
            outcomes: inquired_packages,
        };

        Ok(results)
    }
}

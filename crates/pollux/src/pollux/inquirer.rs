// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::analysers::combined::VeracityChecksAnalyser;
use crate::core::interfaces::CrateVeracityAnalysis;
use crate::core::models::EcosystemInquiringResults;
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

    pub async fn scrutinize_most_popular_crates(&self) -> anyhow::Result<EcosystemInquiringResults> {
        let popular_packages = self.popular_crates_fetcher.get_most_popular_crates().await?;

        let mut scrutinized_packages = vec![];
        let mut with_provenance = 0;
        let mut with_reproducibility = 0;

        for package in popular_packages {
            let checks = self.veracity_analyser.execute(&package).await?;

            match (&checks.provenance_evidence, &checks.reproducibility_evidence) {
                (Some(_), Some(_)) => {
                    with_provenance += 1;
                    with_reproducibility += 1;
                },
                (Some(_), None) => {
                    with_provenance += 1;
                },
                (None, Some(_)) => {
                    with_reproducibility += 1;
                },

                (None, None) => {
                    log::info!("[pollux.scrutinizer] not counting crate {}", package);
                },
            }

            scrutinized_packages.push((package, checks));
        }

        let total_packages = scrutinized_packages.len();

        let results = EcosystemInquiringResults {
            percentual_presence_of_provance: (with_provenance / total_packages) as f32,
            percentual_presence_of_reproducibility: (with_reproducibility / total_packages) as f32,
            outcomes: scrutinized_packages,
        };

        Ok(results)
    }
}

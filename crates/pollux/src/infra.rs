// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::{CrateInfo, TruthfulnessEvaluation};

#[allow(dead_code)]
pub struct CratesIOApiClient;

#[allow(dead_code)]
pub struct OssRebuildBucketClient;

#[allow(dead_code)]
pub enum TrustedPublishingEvaluator {
    FromCratesIO(CratesIOApiClient),
    #[cfg(test)]
    Fake(Vec<CrateInfo>),
}

impl TruthfulnessEvaluation for TrustedPublishingEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        match self {
            TrustedPublishingEvaluator::FromCratesIO(_) => {
                println!("{:?}", crate_info);
                Ok(true)
            },
            #[cfg(test)]
            TrustedPublishingEvaluator::Fake(crates) => Ok(crates.contains(crate_info)),
        }
    }
}

#[allow(dead_code)]
pub enum ReproducibleBuildsEvaluator {
    FromOssRebuild(OssRebuildBucketClient),
    #[cfg(test)]
    Fake(Vec<CrateInfo>),
}

impl TruthfulnessEvaluation for ReproducibleBuildsEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        match self {
            ReproducibleBuildsEvaluator::FromOssRebuild(_) => {
                println!("{:?}", crate_info);
                Ok(true)
            },
            #[cfg(test)]
            ReproducibleBuildsEvaluator::Fake(crates) => Ok(crates.contains(crate_info)),
        }
    }
}

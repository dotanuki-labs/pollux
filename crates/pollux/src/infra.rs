// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod cratesio;
mod ossrebuild;

use crate::core::{CrateInfo, TruthfulnessEvaluation};
use crate::infra::cratesio::CratesIOEvaluator;
use crate::infra::ossrebuild::OssRebuildEvaluator;
use reqwest::Client;

pub type HTTPClient = Client;

#[allow(dead_code)]
pub enum TrustedPublishingEvaluator {
    FromCratesIO(CratesIOEvaluator),
    #[cfg(test)]
    Fake(FakeTruthfulnessEvaluator),
}

impl TruthfulnessEvaluation for TrustedPublishingEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        match self {
            TrustedPublishingEvaluator::FromCratesIO(evaluator) => evaluator.evaluate(crate_info).await,
            #[cfg(test)]
            TrustedPublishingEvaluator::Fake(evaluator) => evaluator.evaluate(crate_info).await,
        }
    }
}

#[allow(dead_code)]
pub enum ReproducibleBuildsEvaluator {
    FromOssRebuild(OssRebuildEvaluator),
    #[cfg(test)]
    Fake(Vec<CrateInfo>),
}

#[allow(unused_variables)]
impl TruthfulnessEvaluation for ReproducibleBuildsEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        match self {
            ReproducibleBuildsEvaluator::FromOssRebuild(delegate) => delegate.evaluate(crate_info).await,
            #[cfg(test)]
            ReproducibleBuildsEvaluator::Fake(crates) => Ok(crates.contains(crate_info)),
        }
    }
}

pub mod factories {
    use crate::infra::cratesio::CratesIOEvaluator;
    use crate::infra::ossrebuild::OssRebuildEvaluator;
    use crate::infra::{HTTPClient, ReproducibleBuildsEvaluator, TrustedPublishingEvaluator};
    use reqwest::header;
    use std::sync::{Arc, LazyLock};

    pub static HTTP_CLIENT: LazyLock<Arc<HTTPClient>> = LazyLock::new(|| {
        let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_str(&user_agent).unwrap());

        let client = HTTPClient::builder().default_headers(headers).build().unwrap();
        Arc::new(client)
    });

    static CRATES_IO_API: &str = "https://crates.io";
    static OSS_REBUILD_CRATES_IO_URL: &str = "https://storage.googleapis.com/google-rebuild-attestations/cratesio";

    pub fn trusted_publishing_evaluator() -> TrustedPublishingEvaluator {
        let delegate = CratesIOEvaluator::new(CRATES_IO_API.to_string(), HTTP_CLIENT.clone());
        TrustedPublishingEvaluator::FromCratesIO(delegate)
    }

    pub fn reproducible_builds_evaluator() -> ReproducibleBuildsEvaluator {
        let delegate = OssRebuildEvaluator::new(OSS_REBUILD_CRATES_IO_URL.to_string(), HTTP_CLIENT.clone());
        ReproducibleBuildsEvaluator::FromOssRebuild(delegate)
    }
}

#[cfg(test)]
pub struct FakeTruthfulnessEvaluator(Vec<CrateInfo>);

#[cfg(test)]
impl TruthfulnessEvaluation for FakeTruthfulnessEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        Ok(self.0.contains(crate_info))
    }
}

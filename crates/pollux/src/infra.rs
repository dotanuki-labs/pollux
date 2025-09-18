// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod cratesio;

use crate::core::{CrateInfo, TruthfulnessEvaluation};
use crate::infra::cratesio::CratesIOEvaluator;
use reqwest::Client;

pub type HTTPClient = Client;

#[allow(dead_code)]
pub struct OssRebuildEvaluator;

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
            ReproducibleBuildsEvaluator::FromOssRebuild(_) => Ok(true),
            #[cfg(test)]
            ReproducibleBuildsEvaluator::Fake(crates) => Ok(crates.contains(crate_info)),
        }
    }
}

pub mod factories {
    use crate::infra::cratesio::CratesIOEvaluator;
    use crate::infra::{HTTPClient, TrustedPublishingEvaluator};
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

    pub fn trusted_publishing_evaluator() -> TrustedPublishingEvaluator {
        let delegate = CratesIOEvaluator::new(CRATES_IO_API.to_string(), HTTP_CLIENT.clone());
        TrustedPublishingEvaluator::FromCratesIO(delegate)
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

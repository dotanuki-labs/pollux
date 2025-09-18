// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod cratesio;
mod ossrebuild;

use crate::core::{CrateInfo, VeracityEvaluation};
use crate::infra::cratesio::CratesIOEvaluator;
use crate::infra::ossrebuild::OssRebuildEvaluator;
use reqwest::Client;

pub type HTTPClient = Client;

#[allow(dead_code)]
pub enum CrateProvenanceEvaluator {
    CratesOfficialRegistry(CratesIOEvaluator),
    #[cfg(test)]
    FakeRegistry(FakeTruthfulnessEvaluator),
}

impl VeracityEvaluation for CrateProvenanceEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        match self {
            CrateProvenanceEvaluator::CratesOfficialRegistry(evaluator) => evaluator.evaluate(crate_info).await,
            #[cfg(test)]
            CrateProvenanceEvaluator::FakeRegistry(evaluator) => evaluator.evaluate(crate_info).await,
        }
    }
}

#[allow(dead_code)]
pub enum CrateBuildReproducibilityEvaluator {
    GoogleOssRebuild(OssRebuildEvaluator),
    #[cfg(test)]
    FakeRebuilder(Vec<CrateInfo>),
}

#[allow(unused_variables)]
impl VeracityEvaluation for CrateBuildReproducibilityEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        match self {
            CrateBuildReproducibilityEvaluator::GoogleOssRebuild(delegate) => delegate.evaluate(crate_info).await,
            #[cfg(test)]
            CrateBuildReproducibilityEvaluator::FakeRebuilder(crates) => Ok(crates.contains(crate_info)),
        }
    }
}

pub mod factories {
    use crate::infra::cratesio::CratesIOEvaluator;
    use crate::infra::ossrebuild::OssRebuildEvaluator;
    use crate::infra::{CrateBuildReproducibilityEvaluator, CrateProvenanceEvaluator, HTTPClient};
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

    pub fn provenance_evaluator() -> CrateProvenanceEvaluator {
        let delegate = CratesIOEvaluator::new(CRATES_IO_API.to_string(), HTTP_CLIENT.clone());
        CrateProvenanceEvaluator::CratesOfficialRegistry(delegate)
    }

    pub fn reproducibility_evaluator() -> CrateBuildReproducibilityEvaluator {
        let delegate = OssRebuildEvaluator::new(OSS_REBUILD_CRATES_IO_URL.to_string(), HTTP_CLIENT.clone());
        CrateBuildReproducibilityEvaluator::GoogleOssRebuild(delegate)
    }
}

#[cfg(test)]
pub struct FakeTruthfulnessEvaluator(Vec<CrateInfo>);

#[cfg(test)]
impl VeracityEvaluation for FakeTruthfulnessEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        Ok(self.0.contains(crate_info))
    }
}

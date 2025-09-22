// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod caching;
pub mod cargo;
pub mod cli;
pub mod cratesio;
pub mod ossrebuild;

use crate::core::{CargoPackage, CrateVeracityLevel, VeracityEvaluation};
use crate::infra::caching::DirectoryBased;
use crate::infra::cratesio::CratesIOEvaluator;
use crate::infra::ossrebuild::OssRebuildEvaluator;
use reqwest::header;

use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;

#[cfg(test)]
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

pub type HTTPClient = ClientWithMiddleware;
pub static CRATES_IO_API: &str = "https://crates.io";
pub static OSS_REBUILD_CRATES_IO_URL: &str = "https://storage.googleapis.com/google-rebuild-attestations/cratesio";
pub static MAX_HTTP_RETRY_ATTEMPTS: u32 = 2;

pub static HTTP_CLIENT: LazyLock<Arc<HTTPClient>> = LazyLock::new(|| {
    let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_str(&user_agent).unwrap());

    let base_http_client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(MAX_HTTP_RETRY_ATTEMPTS);

    let retrier_http_client = reqwest_middleware::ClientBuilder::new(base_http_client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    Arc::new(retrier_http_client)
});

pub enum CrateProvenanceEvaluator {
    CratesOfficialRegistry(CratesIOEvaluator),
    #[cfg(test)]
    FakeRegistry(FakeVeracityEvaluator),
}

impl VeracityEvaluation for CrateProvenanceEvaluator {
    async fn evaluate(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        match self {
            CrateProvenanceEvaluator::CratesOfficialRegistry(delegate) => delegate.evaluate(crate_info).await,
            #[cfg(test)]
            CrateProvenanceEvaluator::FakeRegistry(fake) => fake.evaluate(crate_info).await,
        }
    }
}

pub enum CrateBuildReproducibilityEvaluator {
    GoogleOssRebuild(OssRebuildEvaluator),
    #[cfg(test)]
    FakeRebuilder(FakeVeracityEvaluator),
}

impl VeracityEvaluation for CrateBuildReproducibilityEvaluator {
    async fn evaluate(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        match self {
            CrateBuildReproducibilityEvaluator::GoogleOssRebuild(delegate) => delegate.evaluate(crate_info).await,
            #[cfg(test)]
            CrateBuildReproducibilityEvaluator::FakeRebuilder(fake) => fake.evaluate(crate_info).await,
        }
    }
}

pub enum CachedVeracityEvaluator {
    FileSystem(DirectoryBased),
    #[cfg(test)]
    FakeCache(HashMap<String, CrateVeracityLevel>),
}

pub trait VeracityEvaluationStorage {
    fn read(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityLevel>;
    fn save(&self, crate_info: &CargoPackage, veracity_level: CrateVeracityLevel) -> anyhow::Result<()>;
}

impl VeracityEvaluationStorage for CachedVeracityEvaluator {
    fn read(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityLevel> {
        match self {
            CachedVeracityEvaluator::FileSystem(delegate) => delegate.read(crate_info),
            #[cfg(test)]
            CachedVeracityEvaluator::FakeCache(fakes) => Ok(fakes
                .get(&crate_info.name)
                .cloned()
                .unwrap_or(CrateVeracityLevel::NotAvailable)),
        }
    }

    fn save(&self, crate_info: &CargoPackage, veracity_level: CrateVeracityLevel) -> anyhow::Result<()> {
        match self {
            CachedVeracityEvaluator::FileSystem(delegate) => delegate.save(crate_info, veracity_level),
            #[cfg(test)]
            CachedVeracityEvaluator::FakeCache(fakes) => {
                fakes.to_owned().insert(crate_info.name.clone(), veracity_level);
                Ok(())
            },
        }
    }
}

#[cfg(test)]
pub struct FakeVeracityEvaluator(pub Vec<CargoPackage>);

#[cfg(test)]
impl VeracityEvaluation for FakeVeracityEvaluator {
    async fn evaluate(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        Ok(self.0.contains(crate_info))
    }
}

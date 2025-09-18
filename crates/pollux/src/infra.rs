// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

mod caching;
mod cratesio;
mod ossrebuild;

use crate::core::{CrateInfo, CrateVeracityLevel, VeracityEvaluation};
use crate::infra::caching::DirectoryBased;
use crate::infra::cratesio::CratesIOEvaluator;
use crate::infra::ossrebuild::OssRebuildEvaluator;
use reqwest::Client;

#[cfg(test)]
use std::collections::HashMap;

pub type HTTPClient = Client;

#[allow(dead_code)]
pub enum CrateProvenanceEvaluator {
    CratesOfficialRegistry(CratesIOEvaluator),
    #[cfg(test)]
    FakeRegistry(FakeVeracityEvaluator),
}

impl VeracityEvaluation for CrateProvenanceEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        match self {
            CrateProvenanceEvaluator::CratesOfficialRegistry(delegate) => delegate.evaluate(crate_info).await,
            #[cfg(test)]
            CrateProvenanceEvaluator::FakeRegistry(fake) => fake.evaluate(crate_info).await,
        }
    }
}

#[allow(dead_code)]
pub enum CrateBuildReproducibilityEvaluator {
    GoogleOssRebuild(OssRebuildEvaluator),
    #[cfg(test)]
    FakeRebuilder(FakeVeracityEvaluator),
}

#[allow(unused_variables)]
impl VeracityEvaluation for CrateBuildReproducibilityEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        match self {
            CrateBuildReproducibilityEvaluator::GoogleOssRebuild(delegate) => delegate.evaluate(crate_info).await,
            #[cfg(test)]
            CrateBuildReproducibilityEvaluator::FakeRebuilder(fake) => fake.evaluate(crate_info).await,
        }
    }
}

pub enum CachedVeracityEvaluator {
    FileSystem(DirectoryBased),
    #[allow(dead_code)]
    #[cfg(test)]
    FakeCaching(HashMap<String, CrateVeracityLevel>),
}

pub trait VeracityEvaluationStorage {
    fn read(&self, crate_info: &CrateInfo) -> anyhow::Result<CrateVeracityLevel>;
    fn save(&self, crate_info: &CrateInfo, veracity_level: CrateVeracityLevel) -> anyhow::Result<()>;
}

impl VeracityEvaluationStorage for CachedVeracityEvaluator {
    fn read(&self, crate_info: &CrateInfo) -> anyhow::Result<CrateVeracityLevel> {
        match self {
            CachedVeracityEvaluator::FileSystem(delegate) => delegate.read(crate_info),
            #[cfg(test)]
            CachedVeracityEvaluator::FakeCaching(fakes) => Ok(fakes.get(&crate_info.name).cloned().unwrap()),
        }
    }

    fn save(&self, crate_info: &CrateInfo, veracity_level: CrateVeracityLevel) -> anyhow::Result<()> {
        match self {
            CachedVeracityEvaluator::FileSystem(delegate) => delegate.save(crate_info, veracity_level),
            #[cfg(test)]
            CachedVeracityEvaluator::FakeCaching(fakes) => {
                fakes.to_owned().insert(crate_info.name.clone(), veracity_level);
                Ok(())
            },
        }
    }
}

pub mod factories {
    use crate::infra::caching::DirectoryBased;
    use crate::infra::cratesio::CratesIOEvaluator;
    use crate::infra::ossrebuild::OssRebuildEvaluator;
    use crate::infra::{
        CachedVeracityEvaluator, CrateBuildReproducibilityEvaluator, CrateProvenanceEvaluator, HTTPClient,
    };
    use reqwest::header;
    use std::env::home_dir;
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

    pub fn cached_evaluator() -> CachedVeracityEvaluator {
        let home = home_dir().unwrap().join(".pollux");
        let delegate = DirectoryBased::new(home);
        CachedVeracityEvaluator::FileSystem(delegate)
    }

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
pub struct FakeVeracityEvaluator(Vec<CrateInfo>);

#[cfg(test)]
impl VeracityEvaluation for FakeVeracityEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        Ok(self.0.contains(crate_info))
    }
}

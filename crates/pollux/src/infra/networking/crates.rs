// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::infra::networking::http::HTTPClient;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub mod cargo;
pub mod registry;

pub static URL_OFFICIAL_CRATES_REGISTRY: &str = "https://crates.io";

#[derive(Deserialize)]
#[allow(dead_code)]
struct TrustPubData {
    provider: String,
    repository: String,
    run_id: String,
}

#[derive(Deserialize)]
#[allow(unused_variables)]
struct InfoForCrateVersion {
    trustpub_data: Option<TrustPubData>,
}

#[derive(Deserialize)]
#[allow(unused_variables)]
struct CrateVersionDetails {
    version: InfoForCrateVersion,
}

pub struct CratesDotIOClient {
    base_url: String,
    http_client: Arc<HTTPClient>,
    enforced_delay: u64,
}

impl CratesDotIOClient {
    pub fn new(base_url: String, http_client: Arc<HTTPClient>, enforced_delay: u64) -> Self {
        Self {
            base_url,
            http_client,
            enforced_delay,
        }
    }

    pub async fn get_crate_version_details(&self, crate_name: &str, crate_version: &str) -> anyhow::Result<bool> {
        self.honor_cratesio_rate_limit().await;

        let endpoint = format!("{}/api/v1/crates/{}/{}", self.base_url, crate_name, crate_version);

        let crates_details = self
            .http_client
            .get(&endpoint)
            .send()
            .await?
            .error_for_status()?
            .json::<CrateVersionDetails>()
            .await?;

        Ok(crates_details.version.trustpub_data.is_some())
    }

    pub async fn honor_cratesio_rate_limit(&self) {
        sleep(Duration::from_millis(self.enforced_delay)).await
    }
}

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::{CargoPackage, VeracityEvaluation};
use crate::infra::HTTPClient;
use serde::Deserialize;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct TrustPubData {
    provider: String,
    repository: String,
    run_id: String,
}

impl Display for TrustPubData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "provider = {} | repo = {} | run_id = {}",
            self.provider, self.repository, self.run_id
        ))
    }
}

#[derive(Debug, Deserialize)]
struct CratesVersion {
    trustpub_data: Option<TrustPubData>,
}

#[derive(Debug, Deserialize)]
struct DetailsForCrateVersion {
    version: CratesVersion,
}

pub struct CratesIOEvaluator {
    base_url: String,
    http_client: Arc<HTTPClient>,
    enforced_delay: u64,
}

impl CratesIOEvaluator {
    pub fn new(base_url: String, http_client: Arc<HTTPClient>, enforced_delay: u64) -> Self {
        Self {
            base_url,
            http_client,
            enforced_delay,
        }
    }
}

impl VeracityEvaluation for CratesIOEvaluator {
    async fn evaluate(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        sleep(Duration::from_millis(self.enforced_delay)).await;

        let endpoint = format!(
            "{}/api/v1/crates/{}/{}",
            self.base_url, crate_info.name, crate_info.version
        );

        let crates_details = self
            .http_client
            .get(&endpoint)
            .send()
            .await?
            .error_for_status()?
            .json::<DetailsForCrateVersion>()
            .await?;

        if let Some(trustpub_data) = crates_details.version.trustpub_data {
            log::info!(
                "[pollux.evaluator] found provenance for {} : {}",
                crate_info,
                trustpub_data
            );
            return Ok(true);
        };

        log::info!("[pollux.evaluator] provenance not found for {}", crate_info);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{CargoPackage, VeracityEvaluation};
    use crate::infra::cratesio::CratesIOEvaluator;
    use crate::infra::{HTTP_CLIENT, MAX_HTTP_RETRY_ATTEMPTS};
    use assertor::{BooleanAssertion, ResultAssertion};
    use httpmock::{MockServer, Then, When};

    fn responds_with_existing_provenance(crate_name: &str, crate_version: &str) -> impl FnOnce(When, Then) {
        move |when, then| {
            let crate_version_template = r#"
                    {
                      "version": {
                        "id": 1711352,
                        "crate": "<CRATE_NAME>",
                        "num": "<CRATE_VERSION>",
                        "trustpub_data": {
                          "provider": "github",
                          "repository": "elastio/bon",
                          "run_id": "17402178810",
                          "sha": "bbd8b099ea52bf4de18051d012c8113cf0dca23a"
                        }
                      }
                    }
                "#;

            let payload = crate_version_template
                .replace("<CRATE_NAME>", crate_name)
                .replace("<CRATE_VERSION>", crate_version);

            when.method("GET")
                .path(format!("/api/v1/crates/{}/{}", crate_name, crate_version));

            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(payload);
        }
    }

    fn responds_without_provenance(crate_name: &str, crate_version: &str) -> impl FnOnce(When, Then) {
        move |when, then| {
            let crate_version_template = r#"
                    {
                      "version": {
                        "id": 1711352,
                        "crate": "<CRATE_NAME>",
                        "num": "<CRATE_VERSION>",
                        "trustpub_data": null
                      }
                    }
                "#;

            let payload = crate_version_template
                .replace("<CRATE_NAME>", crate_name)
                .replace("<CRATE_VERSION>", crate_version);

            when.method("GET")
                .path(format!("/api/v1/crates/{}/{}", crate_name, crate_version));

            then.status(200)
                .header("content-type", "application/json; charset=UTF-8")
                .body(payload);
        }
    }

    fn responds_without_server_error(crate_name: &str, crate_version: &str) -> impl FnOnce(When, Then) {
        move |when, then| {
            when.method("GET")
                .path(format!("/api/v1/crates/{}/{}", crate_name, crate_version));

            then.status(503)
                .header("content-type", "application/text; charset=UTF-8")
                .body("internal error");
        }
    }

    #[tokio::test]
    async fn should_evaluate_crate_provenance_when_available() {
        let crate_name = "bon";
        let crate_version = "3.7.2";
        let crate_info = CargoPackage::with(crate_name, crate_version);

        let mock_server = MockServer::start();
        let evaluator = CratesIOEvaluator::new(mock_server.base_url(), HTTP_CLIENT.clone(), 10);

        let with_provenance = responds_with_existing_provenance(crate_name, crate_version);
        let mocked = mock_server.mock(with_provenance);

        let evaluation = evaluator.evaluate(&crate_info).await.unwrap();

        mocked.assert();
        assertor::assert_that!(evaluation).is_true()
    }

    #[tokio::test]
    async fn should_evaluate_crate_provenance_when_not_available() {
        let crate_name = "canopus";
        let crate_version = "0.1.1";
        let crate_info = CargoPackage::with(crate_name, crate_version);

        let mock_server = MockServer::start();
        let evaluator = CratesIOEvaluator::new(mock_server.base_url(), HTTP_CLIENT.clone(), 10);

        let without_provenance = responds_without_provenance(crate_name, crate_version);

        let mocked = mock_server.mock(without_provenance);

        let evaluation = evaluator.evaluate(&crate_info).await.unwrap();

        mocked.assert();
        assertor::assert_that!(evaluation).is_false()
    }

    #[tokio::test]
    async fn should_evaluate_provenance_when_server_not_available() {
        let crate_name = "canopus";
        let crate_version = "0.0.1";
        let crate_info = CargoPackage::with(crate_name, crate_version);

        let mock_server = MockServer::start();
        let evaluator = CratesIOEvaluator::new(mock_server.base_url(), HTTP_CLIENT.clone(), 10);

        let not_found = responds_without_server_error(crate_name, crate_version);
        let mocked = mock_server.mock(not_found);

        let evaluation = evaluator.evaluate(&crate_info).await;

        mocked.assert_hits(MAX_HTTP_RETRY_ATTEMPTS as usize + 1);
        assertor::assert_that!(evaluation).is_err()
    }
}

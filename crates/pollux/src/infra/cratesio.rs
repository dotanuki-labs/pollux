// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::{CrateInfo, TruthfulnessEvaluation};
use crate::infra::HTTPClient;
use serde::Deserialize;
use std::fmt::Display;
use std::sync::Arc;

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
}

impl CratesIOEvaluator {
    pub fn new(base_url: String, http_client: Arc<HTTPClient>) -> Self {
        Self { base_url, http_client }
    }
}

impl TruthfulnessEvaluation for CratesIOEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
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
            log::info!("Found provenance for {} : {}", crate_info, trustpub_data);
            return Ok(true);
        };

        log::info!("Provenance not found for {}", crate_info);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{CrateInfo, TruthfulnessEvaluation};
    use crate::infra::cratesio::CratesIOEvaluator;
    use crate::infra::factories;
    use assertor::BooleanAssertion;
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

    #[tokio::test]
    async fn should_evaluate_provenance_when_available() {
        let mock_server = MockServer::start();
        let evaluator = CratesIOEvaluator::new(mock_server.base_url(), factories::HTTP_CLIENT.clone());

        let response_with_provenance = responds_with_existing_provenance("bon", "3.7.2");

        let mocked = mock_server.mock(response_with_provenance);
        let crate_info = CrateInfo::new("bon".to_string(), "3.7.2".to_string());

        let evaluation = evaluator.evaluate(&crate_info).await.unwrap();

        mocked.assert();
        assertor::assert_that!(evaluation).is_true()
    }

    #[tokio::test]
    async fn should_evaluate_provenance_when_not_available() {
        let mock_server = MockServer::start();
        let evaluator = CratesIOEvaluator::new(mock_server.base_url(), factories::HTTP_CLIENT.clone());

        let response_without_provenance = responds_without_provenance("canopus", "0.1.1");

        let mocked = mock_server.mock(response_without_provenance);

        let crate_info = CrateInfo::new("canopus".to_string(), "0.1.1".to_string());

        let evaluation = evaluator.evaluate(&crate_info).await.unwrap();

        mocked.assert();
        assertor::assert_that!(evaluation).is_false()
    }
}

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::interfaces::VeracityFactorEvaluation;
use crate::core::models::CargoPackage;
use crate::infra::networking::crates::registry::CratesDotIOClient;

pub mod registry;
pub mod resolvers;
pub mod tarballs;

pub struct OfficialCratesRegistryEvaluator {
    cratesio_client: CratesDotIOClient,
}

impl OfficialCratesRegistryEvaluator {
    pub fn new(cratesio_client: CratesDotIOClient) -> Self {
        Self { cratesio_client }
    }
}

impl VeracityFactorEvaluation for OfficialCratesRegistryEvaluator {
    async fn evaluate(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        let has_provenance = self
            .cratesio_client
            .get_crate_version_details(crate_info.name.as_str(), crate_info.version.as_str())
            .await?;

        if has_provenance {
            log::info!("[pollux.evaluator] found provenance for {} ", crate_info,);
            return Ok(has_provenance);
        };

        log::info!("[pollux.evaluator] provenance not found for {}", crate_info);
        Ok(has_provenance)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::interfaces::VeracityFactorEvaluation;
    use crate::core::models::CargoPackage;
    use crate::infra::networking::crates::OfficialCratesRegistryEvaluator;
    use crate::infra::networking::crates::registry::CratesDotIOClient;
    use crate::infra::networking::http::{HTTP_CLIENT, MAX_HTTP_RETRY_ATTEMPTS};
    use assertor::{BooleanAssertion, ResultAssertion};
    use httpmock::{MockServer, Then, When};

    static SMALL_DELAY_FOR_RATE_LIMITING: u64 = 10;

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
        let cratesio_client = CratesDotIOClient::new(
            mock_server.base_url(),
            HTTP_CLIENT.clone(),
            SMALL_DELAY_FOR_RATE_LIMITING,
        );
        let evaluator = OfficialCratesRegistryEvaluator::new(cratesio_client);

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
        let cratesio_client = CratesDotIOClient::new(
            mock_server.base_url(),
            HTTP_CLIENT.clone(),
            SMALL_DELAY_FOR_RATE_LIMITING,
        );
        let evaluator = OfficialCratesRegistryEvaluator::new(cratesio_client);

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
        let cratesio_client = CratesDotIOClient::new(
            mock_server.base_url(),
            HTTP_CLIENT.clone(),
            SMALL_DELAY_FOR_RATE_LIMITING,
        );
        let evaluator = OfficialCratesRegistryEvaluator::new(cratesio_client);

        let not_found = responds_without_server_error(crate_name, crate_version);
        let mocked = mock_server.mock(not_found);

        let evaluation = evaluator.evaluate(&crate_info).await;

        mocked.assert_calls(MAX_HTTP_RETRY_ATTEMPTS as usize + 1);
        assertor::assert_that!(evaluation).is_err()
    }
}

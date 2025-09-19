// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::{CrateInfo, VeracityEvaluation};
use crate::infra::HTTPClient;
use anyhow::bail;
use reqwest::StatusCode;
use std::sync::Arc;

pub struct OssRebuildEvaluator {
    base_url: String,
    http_client: Arc<HTTPClient>,
}

impl OssRebuildEvaluator {
    pub fn new(base_url: String, http_client: Arc<HTTPClient>) -> Self {
        Self { base_url, http_client }
    }
}

impl VeracityEvaluation for OssRebuildEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool> {
        let endpoint = format!(
            "{}/{}/{}/{}-{}.crate/rebuild.intoto.jsonl",
            self.base_url, crate_info.name, crate_info.version, crate_info.name, crate_info.version
        );

        let response = self.http_client.head(&endpoint).send().await?;

        if response.status() == StatusCode::OK {
            log::info!("[pollux.evaluator] found reproduced build for {}", crate_info);
            return Ok(true);
        }

        if response.status() == StatusCode::NOT_FOUND {
            log::info!("[pollux.evaluator] reproduced build not found for {}", crate_info);
            return Ok(false);
        }

        bail!(
            "pollux.evaluator : cannot fetch information from oss-rebuild (HTTP status = {})",
            response.status()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{CrateInfo, VeracityEvaluation};
    use crate::infra::factories;
    use crate::infra::ossrebuild::OssRebuildEvaluator;
    use assertor::{BooleanAssertion, ResultAssertion};
    use httpmock::MockServer;

    #[tokio::test]
    async fn should_evaluate_rebuild_when_available() {
        let mock_server = MockServer::start();
        let evaluator = OssRebuildEvaluator::new(mock_server.base_url(), factories::HTTP_CLIENT.clone());

        let name = "castaway";
        let version = "0.2.2";

        let crate_info = CrateInfo::new(name.to_string(), version.to_string());
        let endpoint = format!("/{}/{}/{}-{}.crate/rebuild.intoto.jsonl", name, version, name, version);

        let mocked = mock_server.mock(|when, then| {
            when.method("HEAD").path(endpoint);

            then.status(200).header("content-type", "text/plain; charset=UTF-8");
        });

        let evaluation = evaluator.evaluate(&crate_info).await.unwrap();

        mocked.assert();
        assertor::assert_that!(evaluation).is_true()
    }

    #[tokio::test]
    async fn should_evaluate_rebuild_when_not_available() {
        let mock_server = MockServer::start();
        let evaluator = OssRebuildEvaluator::new(mock_server.base_url(), factories::HTTP_CLIENT.clone());

        let name = "castaway";
        let version = "0.1.0";

        let crate_info = CrateInfo::new(name.to_string(), version.to_string());

        let endpoint = format!("/{}/{}/{}-{}.crate/rebuild.intoto.jsonl", name, version, name, version);

        let mocked = mock_server.mock(|when, then| {
            when.method("HEAD").path(endpoint);

            then.status(404)
                .header("content-type", "text/plain; charset=UTF-8")
                .body("not found");
        });

        let evaluation = evaluator.evaluate(&crate_info).await.unwrap();

        mocked.assert();
        assertor::assert_that!(evaluation).is_false()
    }

    #[tokio::test]
    async fn should_not_evaluate_rebuild_when_with_different_status_code() {
        let mock_server = MockServer::start();
        let evaluator = OssRebuildEvaluator::new(mock_server.base_url(), factories::HTTP_CLIENT.clone());

        let name = "castaway";
        let version = "0.2.4";

        let crate_info = CrateInfo::new(name.to_string(), version.to_string());

        let endpoint = format!("/{}/{}/{}-{}.crate/rebuild.intoto.jsonl", name, version, name, version);

        let mocked = mock_server.mock(|when, then| {
            when.method("HEAD").path(endpoint);

            then.status(503)
                .header("content-type", "text/plain; charset=UTF-8")
                .body("internal server error");
        });

        let evaluation = evaluator.evaluate(&crate_info).await;

        mocked.assert();
        assertor::assert_that!(evaluation).is_err()
    }
}

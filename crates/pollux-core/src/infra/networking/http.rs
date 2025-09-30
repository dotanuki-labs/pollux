// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use reqwest::header;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

pub type HTTPClient = ClientWithMiddleware;

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

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::utils::BuildEnvironment::{CI, Local};
use std::env;

static CALLINECTES_DOCKER_IMAGE: &str = "ghcr.io/dotanuki-labs/callinectes:latest";
static CALLINECTES_DOCKER_DIGEST: &str = "3a5252044164b7aca1ebb7d23c850177a7c280fb91d277d2b88dcdb4b9d03571";

pub enum BuildEnvironment {
    CI,
    Local,
}

pub fn evaluate_build_environment() -> BuildEnvironment {
    match env::var("CI") {
        Ok(_) => CI,
        Err(_) => Local,
    }
}

pub fn docker_execution_arguments() -> (String, String) {
    let docker_package = format!("{CALLINECTES_DOCKER_IMAGE}@sha256:{CALLINECTES_DOCKER_DIGEST}");
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let pwd = current_dir
        .to_str()
        .expect("Failed to convert current directory to string");
    (format!("{pwd}:/usr/src"), docker_package)
}

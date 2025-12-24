// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::utils::BuildEnvironment::{CI, Local};
use std::env;

static CALLINECTES_DOCKER_IMAGE: &str = "ghcr.io/dotanuki-labs/callinectes:latest";

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
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let pwd = current_dir
        .to_str()
        .expect("Failed to convert current directory to string");
    (format!("{pwd}:/usr/src"), CALLINECTES_DOCKER_IMAGE.to_string())
}

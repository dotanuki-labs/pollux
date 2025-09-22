// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::CombinedVeracityEvaluator;
use crate::infra::caching::DirectoryBased;
use crate::infra::cargo::RustProjectDependenciesResolver;
use crate::infra::cratesio::CratesIOEvaluator;
use crate::infra::ossrebuild::OssRebuildEvaluator;
use crate::infra::{
    CRATES_IO_API, CachedVeracityEvaluator, CrateBuildReproducibilityEvaluator, CrateProvenanceEvaluator, HTTP_CLIENT,
    OSS_REBUILD_CRATES_IO_URL,
};
use crate::pollux::{Pollux, PolluxExecutor, PolluxTask};
use std::env::home_dir;
use std::path::PathBuf;

pub static CRATESIO_MILLIS_TO_WAIT_AFTER_RATE_LIMITED: u64 = 1100;

fn cached_evaluator() -> CachedVeracityEvaluator {
    let cache_folder = match home_dir() {
        None => PathBuf::from("/var/cache"),
        Some(dir) => dir.join(".pollux"),
    };

    let delegate = DirectoryBased::new(cache_folder);
    CachedVeracityEvaluator::FileSystem(delegate)
}

fn provenance_evaluator() -> CrateProvenanceEvaluator {
    let delegate = CratesIOEvaluator::new(
        CRATES_IO_API.to_string(),
        HTTP_CLIENT.clone(),
        CRATESIO_MILLIS_TO_WAIT_AFTER_RATE_LIMITED,
    );
    CrateProvenanceEvaluator::CratesOfficialRegistry(delegate)
}

fn reproducibility_evaluator() -> CrateBuildReproducibilityEvaluator {
    let delegate = OssRebuildEvaluator::new(OSS_REBUILD_CRATES_IO_URL.to_string(), HTTP_CLIENT.clone());
    CrateBuildReproducibilityEvaluator::GoogleOssRebuild(delegate)
}

fn veracity_evaluator() -> CombinedVeracityEvaluator {
    CombinedVeracityEvaluator::new(cached_evaluator(), provenance_evaluator(), reproducibility_evaluator())
}

pub fn create_pollux(task: PolluxTask) -> Pollux {
    match task {
        PolluxTask::EvaluateRustProject(project_path) => {
            let dependencies_resolver = RustProjectDependenciesResolver::new(project_path);
            let pollux_executor = PolluxExecutor::new(veracity_evaluator());
            Pollux::new(dependencies_resolver, pollux_executor)
        },
    }
}

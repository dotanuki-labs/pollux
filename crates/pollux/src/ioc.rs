// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::evaluators::combined::CombinedVeracityEvaluator;
use crate::core::evaluators::standalone::{
    BuildReproducibilityEvaluator, CachedExecutionEvaluator, CrateProvenanceEvaluator,
};
use crate::infra::caching::filesystem::DirectoryBased;
use crate::infra::networking::crates::CratesDotIOClient;
use crate::infra::networking::crates::cargo::{CrateArchiveDownloader, DependenciesResolver};
use crate::infra::networking::crates::registry::OfficialCratesRegistryEvaluator;
use crate::infra::networking::http::HTTP_CLIENT;
use crate::infra::networking::ossrebuild::OssRebuildEvaluator;
use crate::infra::networking::{crates, ossrebuild};
use crate::pollux::actors::PolluxEvaluatorActor;
use crate::pollux::{Pollux, PolluxTask};
use std::env::home_dir;
use std::path::PathBuf;

pub static CRATESIO_MILLIS_TO_WAIT_AFTER_RATE_LIMITED: u64 = 1100;

fn cache_folder() -> PathBuf {
    match home_dir() {
        None => PathBuf::from("/var/cache"),
        Some(dir) => dir.join(".pollux"),
    }
}

fn cached_evaluator() -> CachedExecutionEvaluator {
    let delegate = DirectoryBased::new(cache_folder());
    CachedExecutionEvaluator::FileSystem(delegate)
}

fn cratesio_client() -> CratesDotIOClient {
    CratesDotIOClient::new(
        crates::URL_OFFICIAL_CRATES_REGISTRY.to_string(),
        HTTP_CLIENT.clone(),
        CRATESIO_MILLIS_TO_WAIT_AFTER_RATE_LIMITED,
    )
}

fn provenance_evaluator() -> CrateProvenanceEvaluator {
    let delegate = OfficialCratesRegistryEvaluator::new(cratesio_client());
    CrateProvenanceEvaluator::CratesOfficialRegistry(delegate)
}

fn reproducibility_evaluator() -> BuildReproducibilityEvaluator {
    let delegate = OssRebuildEvaluator::new(ossrebuild::URL_OSS_REBUILD_CRATES.to_string(), HTTP_CLIENT.clone());
    BuildReproducibilityEvaluator::GoogleOssRebuild(delegate)
}

fn veracity_evaluator() -> CombinedVeracityEvaluator {
    CombinedVeracityEvaluator::new(cached_evaluator(), provenance_evaluator(), reproducibility_evaluator())
}

pub fn create_pollux(task: PolluxTask) -> Pollux {
    match task {
        PolluxTask::EvaluateRustProject(project_root) => {
            let dependencies_resolver = DependenciesResolver::LocalRustProject { project_root };
            let pollux_executor = PolluxEvaluatorActor::new(veracity_evaluator());
            Pollux::new(dependencies_resolver, pollux_executor)
        },
        PolluxTask::EvaluateRustCrate(cargo_package) => {
            let crate_downloader = CrateArchiveDownloader::new(cratesio_client(), cache_folder(), cargo_package);
            let dependencies_resolver = DependenciesResolver::StandaloneCargoPackage { crate_downloader };
            let pollux_executor = PolluxEvaluatorActor::new(veracity_evaluator());
            Pollux::new(dependencies_resolver, pollux_executor)
        },
    }
}

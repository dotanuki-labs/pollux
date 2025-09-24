// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::evaluators::combined::CombinedVeracityEvaluator;
use crate::core::evaluators::standalone::{
    BuildReproducibilityEvaluator, CachedExecutionEvaluator, CrateProvenanceEvaluator,
};
use crate::infra::caching::CacheManager;
use crate::infra::caching::filesystem::DirectoryBased;
use crate::infra::networking::crates::CratesDotIOClient;
use crate::infra::networking::crates::cargo::{CrateArchiveDownloader, DependenciesResolver};
use crate::infra::networking::crates::registry::OfficialCratesRegistryEvaluator;
use crate::infra::networking::http::HTTP_CLIENT;
use crate::infra::networking::ossrebuild::OssRebuildEvaluator;
use crate::infra::networking::{crates, ossrebuild};
use crate::pollux::Pollux;
use crate::pollux::actors::PolluxEvaluatorActor;

pub static MILLIS_TO_WAIT_AFTER_RATE_LIMITED: u64 = 1100;

fn cached_evaluator() -> CachedExecutionEvaluator {
    let delegate = DirectoryBased::new(CacheManager::get());
    CachedExecutionEvaluator::FileSystem(delegate)
}

fn cratesio_client() -> CratesDotIOClient {
    CratesDotIOClient::new(
        crates::URL_OFFICIAL_CRATES_REGISTRY.to_string(),
        HTTP_CLIENT.clone(),
        MILLIS_TO_WAIT_AFTER_RATE_LIMITED,
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

fn pollux_evaluator() -> PolluxEvaluatorActor {
    PolluxEvaluatorActor::new(veracity_evaluator())
}

fn dependencies_resolver() -> DependenciesResolver {
    let downloader = CrateArchiveDownloader::new(cratesio_client(), CacheManager::get());
    DependenciesResolver::new(downloader)
}

pub fn create_pollux() -> Pollux {
    Pollux::new(CacheManager::get(), dependencies_resolver(), pollux_evaluator)
}

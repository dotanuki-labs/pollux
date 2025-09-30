// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::analysers::combined::VeracityChecksAnalyser;
use crate::core::analysers::standalone::{BuildReproducibilityChecker, CachedDataChecker, CrateProvenanceChecker};
use crate::infra::caching::CacheManager;
use crate::infra::caching::analysis::AnalysedPackagesCache;
use crate::infra::cli::reporter::ConsoleReporter;
use crate::infra::networking::crates::OfficialCratesRegistryChecker;
use crate::infra::networking::crates::registry::CratesDotIOClient;
use crate::infra::networking::crates::resolvers::DependenciesResolver;
use crate::infra::networking::crates::tarballs::CrateArchiveDownloader;
use crate::infra::networking::http::HTTP_CLIENT;
use crate::infra::networking::ossrebuild::OssRebuildChecker;
use crate::infra::networking::{crates, ossrebuild};
use crate::pollux::Pollux;
use crate::pollux::analyser::PolluxAnalyser;
use crate::pollux::checker::PolluxChecker;
use crate::pollux::cleaner::PolluxCleaner;

pub static MILLIS_TO_WAIT_AFTER_RATE_LIMITED: u64 = 1100;

fn cratesio_client() -> CratesDotIOClient {
    CratesDotIOClient::new(
        crates::registry::URL_OFFICIAL_CRATES_REGISTRY.to_string(),
        HTTP_CLIENT.clone(),
        MILLIS_TO_WAIT_AFTER_RATE_LIMITED,
    )
}

fn cached_checker() -> CachedDataChecker {
    let delegate = AnalysedPackagesCache::new(CacheManager::get());
    CachedDataChecker::FileSystem(delegate)
}

fn provenance_checker() -> CrateProvenanceChecker {
    let delegate = OfficialCratesRegistryChecker::new(cratesio_client());
    CrateProvenanceChecker::CratesOfficialRegistry(delegate)
}

fn reproducibility_checker() -> BuildReproducibilityChecker {
    let delegate = OssRebuildChecker::new(ossrebuild::URL_OSS_REBUILD_CRATES.to_string(), HTTP_CLIENT.clone());
    BuildReproducibilityChecker::GoogleOssRebuild(delegate)
}

fn veracity_analyser() -> VeracityChecksAnalyser {
    VeracityChecksAnalyser::new(cached_checker(), provenance_checker(), reproducibility_checker())
}

fn dependencies_resolver() -> DependenciesResolver {
    let downloader = CrateArchiveDownloader::new(cratesio_client(), CacheManager::get());
    DependenciesResolver::new(downloader)
}

fn pollux_analyser() -> PolluxAnalyser {
    PolluxAnalyser::new(dependencies_resolver(), veracity_analyser())
}

fn pollux_checker() -> PolluxChecker {
    PolluxChecker::new(veracity_analyser())
}

fn pollux_cleaner() -> PolluxCleaner {
    PolluxCleaner::new(CacheManager::get())
}

pub fn create_pollux(turnoff_colors: bool) -> Pollux {
    Pollux::new(
        pollux_cleaner(),
        pollux_analyser(),
        pollux_checker(),
        ConsoleReporter::new(turnoff_colors),
    )
}

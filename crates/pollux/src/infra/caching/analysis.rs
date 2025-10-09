// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::interfaces::AnalyzedDataStorage;
use crate::core::models::{CargoPackage, CrateVeracityChecks};
use crate::infra::caching::CacheManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

static VERACITY_CHECKS_FILE_NAME: &str = "checks.json";

#[derive(Debug, Serialize, Deserialize)]
struct CachedVeracityInfo {
    crate_purl: String,
    trusted_publishing: Option<String>,
    reproducibility: Option<String>,
}

pub struct AnalysedPackagesCache {
    cache_manager: CacheManager,
}

impl AnalysedPackagesCache {
    pub fn new(cache_manager: CacheManager) -> Self {
        Self { cache_manager }
    }

    fn data_dir(&self, crate_info: &CargoPackage) -> PathBuf {
        self.cache_manager
            .analysis_cache_dir()
            .join(&crate_info.name)
            .join(&crate_info.version)
    }
}

impl AnalyzedDataStorage for AnalysedPackagesCache {
    fn retrieve(&self, crate_info: &CargoPackage) -> anyhow::Result<Option<CrateVeracityChecks>> {
        let destination_dir = self.data_dir(crate_info);
        let cache_file = destination_dir.join(VERACITY_CHECKS_FILE_NAME);

        if !cache_file.exists() {
            log::info!("[pollux.cache] {:?} not found", destination_dir);
            return Ok(None);
        }

        log::info!("[pollux.cache] cache hit at {:?}", cache_file);
        let serialized = std::fs::read(cache_file)?;
        let info: CachedVeracityInfo = serde_json::from_slice(&serialized)?;
        let checks = CrateVeracityChecks::new(
            info.trusted_publishing
                .map(|url| Url::from_str(&url).expect("cannot parse cache url")),
            info.reproducibility
                .map(|url| Url::from_str(&url).expect("cannot parse cache url")),
        );
        Ok(Some(checks))
    }

    fn save(&self, crate_info: &CargoPackage, checks: CrateVeracityChecks) -> anyhow::Result<()> {
        let destination_dir = self.data_dir(crate_info);
        let cache_file = destination_dir.join(VERACITY_CHECKS_FILE_NAME);

        if !destination_dir.exists() {
            std::fs::create_dir_all(&destination_dir).expect("cannot create cache folder");
            log::info!("[pollux.cache] {:?} created", destination_dir);
        }

        let cached_veracity = CachedVeracityInfo {
            crate_purl: crate_info.to_string(),
            trusted_publishing: checks.trusted_publishing_evidence.map(|url| url.to_string()),
            reproducibility: checks.reproducibility_evidence.map(|url| url.to_string()),
        };

        let serialized = serde_json::to_vec(&cached_veracity)?;
        std::fs::write(destination_dir.join(VERACITY_CHECKS_FILE_NAME), serialized)?;
        log::info!("[pollux.cache] {:?} saved", cache_file);
        Ok(())
    }
}

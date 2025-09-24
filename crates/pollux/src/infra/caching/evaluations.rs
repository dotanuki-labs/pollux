// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::interfaces::VeracityEvaluationStorage;
use crate::core::models::{CargoPackage, CrateVeracityLevel};
use crate::infra::caching::CacheManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

static VERACITY_CHECKS_FILE_NAME: &str = "checks.json";

#[derive(Debug, Serialize, Deserialize)]
struct CachedVeracityInfo {
    crate_purl: String,
    provenance: bool,
    reproducibility: bool,
}

pub struct VeracityEvaluationsCache {
    cache_manager: CacheManager,
}

impl VeracityEvaluationsCache {
    pub fn new(cache_manager: CacheManager) -> Self {
        Self { cache_manager }
    }

    fn data_dir(&self, crate_info: &CargoPackage) -> PathBuf {
        self.cache_manager
            .evaluations_cache_dir()
            .join(&crate_info.name)
            .join(&crate_info.version)
    }
}

impl VeracityEvaluationStorage for VeracityEvaluationsCache {
    fn retrieve_evaluation(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityLevel> {
        let destination_dir = self.data_dir(crate_info);
        let cache_file = destination_dir.join(VERACITY_CHECKS_FILE_NAME);

        if !cache_file.exists() {
            log::info!("[pollux.cache] {:?} not found", destination_dir);
            return Ok(CrateVeracityLevel::NotAvailable);
        }

        log::info!("[pollux.cache] cache hit at {:?}", cache_file);
        let serialized = std::fs::read(cache_file)?;
        let info: CachedVeracityInfo = serde_json::from_slice(&serialized)?;
        let veracity_level = CrateVeracityLevel::from_booleans(info.provenance, info.reproducibility);
        Ok(veracity_level)
    }

    fn save_evaluation(&self, crate_info: &CargoPackage, veracity_level: CrateVeracityLevel) -> anyhow::Result<()> {
        let destination_dir = self.data_dir(crate_info);
        let cache_file = destination_dir.join(VERACITY_CHECKS_FILE_NAME);

        if !destination_dir.exists() {
            std::fs::create_dir_all(&destination_dir).expect("cannot create cache folder");
            log::info!("[pollux.cache] {:?} created", destination_dir);
        }

        let (attested, reproduced) = veracity_level.to_booleans();

        let cached_veracity = CachedVeracityInfo {
            crate_purl: crate_info.to_string(),
            provenance: attested,
            reproducibility: reproduced,
        };

        let serialized = serde_json::to_vec(&cached_veracity)?;
        std::fs::write(destination_dir.join(VERACITY_CHECKS_FILE_NAME), serialized)?;
        log::info!("[pollux.cache] {:?} saved", cache_file);
        Ok(())
    }
}

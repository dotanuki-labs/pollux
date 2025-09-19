// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::{CargoPackage, CrateVeracityLevel};
use crate::infra::VeracityEvaluationStorage;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

static VERACITY_FILE_NAME: &str = "veracity-checks.json";

#[derive(Debug, Serialize, Deserialize)]
struct CachedVeracityInfo {
    crate_purl: String,
    provenance: bool,
    reproducibility: bool,
}

pub struct DirectoryBased {
    cache_dir: PathBuf,
}

impl DirectoryBased {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    fn data_dir(&self, crate_info: &CargoPackage) -> PathBuf {
        self.cache_dir
            .join("cache")
            .join(&crate_info.name)
            .join(&crate_info.version)
    }
}

impl VeracityEvaluationStorage for DirectoryBased {
    fn read(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityLevel> {
        let destination_dir = self.data_dir(crate_info);
        let cache_file = destination_dir.join(VERACITY_FILE_NAME);

        if !cache_file.exists() {
            log::info!("[pollux.cache] {:?} not found", destination_dir);
            return Ok(CrateVeracityLevel::NotAvailable);
        }

        log::info!("[pollux.cache] cache hit at {:?} created", cache_file);
        let serialized = std::fs::read(cache_file)?;
        let info: CachedVeracityInfo = serde_json::from_slice(&serialized)?;
        let veracity_level = CrateVeracityLevel::from_booleans(info.provenance, info.reproducibility);
        Ok(veracity_level)
    }

    fn save(&self, crate_info: &CargoPackage, veracity_level: CrateVeracityLevel) -> anyhow::Result<()> {
        let destination_dir = self.data_dir(crate_info);
        let cache_file = destination_dir.join(VERACITY_FILE_NAME);

        if !destination_dir.exists() {
            std::fs::create_dir_all(&destination_dir).expect("cannot create cache at $HOME");
            log::info!("[pollux.cache] {:?} created", destination_dir);
        }

        let (attested, reproduced) = veracity_level.to_booleans();

        let cached_veracity = CachedVeracityInfo {
            crate_purl: crate_info.to_string(),
            provenance: attested,
            reproducibility: reproduced,
        };

        let serialized = serde_json::to_vec(&cached_veracity)?;
        std::fs::write(destination_dir.join(VERACITY_FILE_NAME), serialized)?;
        log::info!("[pollux.cache] {:?} saved", cache_file);
        Ok(())
    }
}

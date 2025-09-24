// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use std::env::home_dir;
use std::path::{Path, PathBuf};

pub mod filesystem;

static CACHE_FOLDER_EVALUATIONS: &str = "evaluations";
static CACHE_FOLDER_PACKAGES: &str = "packages";

pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn get() -> Self {
        let cache_dir = match home_dir() {
            None => PathBuf::from("/var/cache/.pollux"),
            Some(dir) => dir.join(".pollux"),
        };
        Self { cache_dir }
    }

    pub fn evaluations_cache_dir(&self) -> PathBuf {
        self.cache_dir.join(CACHE_FOLDER_EVALUATIONS)
    }

    pub fn packages_cache_dir(&self) -> PathBuf {
        self.cache_dir.join(CACHE_FOLDER_PACKAGES)
    }

    pub fn cleanup_packages(&self) {
        self.cleanup(self.packages_cache_dir().as_path());
    }

    pub fn cleanup_evaluations(&self) {
        self.cleanup(self.evaluations_cache_dir().as_path());
    }

    pub fn cleanup_all(&self) {
        self.cleanup(self.cache_dir.as_path());
    }

    fn cleanup(&self, target_folder: &Path) {
        match std::fs::remove_dir_all(target_folder) {
            Ok(_) => log::info!("[pollux.cache] removed {:?}", target_folder),
            Err(_) => log::error!("[pollux.cache] cannot remove : {:?}", target_folder),
        }
    }
}

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use std::env::home_dir;
use std::path::{Path, PathBuf};

pub mod analysis;

static CACHE_FOLDER_ANALYSED: &str = "analysed";
static CACHE_FOLDER_PACKAGES: &str = "packages";
static TEMP_DOWNLOADS_FOLDER: &str = "downloads";

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

    pub fn temporary_downloads_dir(&self) -> PathBuf {
        self.cache_dir.join(TEMP_DOWNLOADS_FOLDER)
    }

    pub fn analysis_cache_dir(&self) -> PathBuf {
        self.cache_dir.join(CACHE_FOLDER_ANALYSED)
    }

    pub fn packages_cache_dir(&self) -> PathBuf {
        self.cache_dir.join(CACHE_FOLDER_PACKAGES)
    }

    pub fn cleanup_cached_packages_sources(&self) {
        self.cleanup(self.packages_cache_dir().as_path());
    }

    pub fn cleanup_cached_analysis(&self) {
        self.cleanup(self.analysis_cache_dir().as_path());
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

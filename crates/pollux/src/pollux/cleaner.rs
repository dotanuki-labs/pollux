// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::infra::caching::CacheManager;

pub struct PolluxCleaner {
    cache_manager: CacheManager,
}

impl PolluxCleaner {
    pub fn new(cache_manager: CacheManager) -> Self {
        Self { cache_manager }
    }

    pub fn cleanup_analysed_data(&self) -> anyhow::Result<()> {
        self.cache_manager.cleanup_cached_analysis();
        println!("Cached analysis removed with success!");
        Ok(())
    }

    pub fn cleanup_package_sources(&self) -> anyhow::Result<()> {
        self.cache_manager.cleanup_cached_packages_sources();
        println!("Cached package sources removed with success!");
        Ok(())
    }

    pub fn cleanup_everything(&self) -> anyhow::Result<()> {
        self.cache_manager.cleanup_all();
        println!("All caches removed with success!");
        Ok(())
    }
}

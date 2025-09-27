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

    pub fn cleanup_cached_evaluations(&self) -> anyhow::Result<()> {
        self.cache_manager.cleanup_evaluations();
        println!("Cached evaluations removed with success!");
        Ok(())
    }

    pub fn cleanup_cached_packages(&self) -> anyhow::Result<()> {
        self.cache_manager.cleanup_packages();
        println!("Cached packages removed with success!");
        Ok(())
    }

    pub fn cleanup_everything(&self) -> anyhow::Result<()> {
        self.cache_manager.cleanup_all();
        println!("All caches removed with success!");
        Ok(())
    }
}

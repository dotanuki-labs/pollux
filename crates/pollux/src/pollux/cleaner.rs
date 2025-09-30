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

    pub fn cleanup_analysed_data(&self) {
        self.cache_manager.cleanup_cached_analysis();
    }

    pub fn cleanup_package_sources(&self) {
        self.cache_manager.cleanup_cached_packages_sources();
    }

    pub fn cleanup_everything(&self) {
        self.cache_manager.cleanup_all();
    }
}

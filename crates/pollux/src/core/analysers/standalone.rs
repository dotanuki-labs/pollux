// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::interfaces::{AnalyzedDataStorage, VeracityFactorCheck};
use crate::core::models::{CargoPackage, CrateVeracityLevel};
use crate::infra::caching::analysis::AnalysedPackagesCache;
use crate::infra::networking::crates::OfficialCratesRegistryChecker;
use crate::infra::networking::ossrebuild::OssRebuildChecker;
#[cfg(test)]
use std::collections::HashMap;

pub enum CrateProvenanceChecker {
    CratesOfficialRegistry(OfficialCratesRegistryChecker),
    #[cfg(test)]
    FakeRegistry(FakeVeracityChecker),
}

impl VeracityFactorCheck for CrateProvenanceChecker {
    async fn execute(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        match self {
            CrateProvenanceChecker::CratesOfficialRegistry(delegate) => delegate.execute(crate_info).await,
            #[cfg(test)]
            CrateProvenanceChecker::FakeRegistry(fake) => fake.execute(crate_info).await,
        }
    }
}

pub enum BuildReproducibilityChecker {
    GoogleOssRebuild(OssRebuildChecker),
    #[cfg(test)]
    FakeRebuilder(FakeVeracityChecker),
}

impl VeracityFactorCheck for BuildReproducibilityChecker {
    async fn execute(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        match self {
            BuildReproducibilityChecker::GoogleOssRebuild(delegate) => delegate.execute(crate_info).await,
            #[cfg(test)]
            BuildReproducibilityChecker::FakeRebuilder(fake) => fake.execute(crate_info).await,
        }
    }
}

pub enum CachedDataChecker {
    FileSystem(AnalysedPackagesCache),
    #[cfg(test)]
    FakeCache(HashMap<String, CrateVeracityLevel>),
}

impl AnalyzedDataStorage for CachedDataChecker {
    fn retrieve(&self, crate_info: &CargoPackage) -> anyhow::Result<Option<CrateVeracityLevel>> {
        match self {
            CachedDataChecker::FileSystem(delegate) => delegate.retrieve(crate_info),
            #[cfg(test)]
            CachedDataChecker::FakeCache(fakes) => Ok(fakes.get(&crate_info.name).cloned()),
        }
    }

    fn save(&self, crate_info: &CargoPackage, veracity_level: CrateVeracityLevel) -> anyhow::Result<()> {
        match self {
            CachedDataChecker::FileSystem(delegate) => delegate.save(crate_info, veracity_level),
            #[cfg(test)]
            CachedDataChecker::FakeCache(fakes) => {
                fakes.to_owned().insert(crate_info.name.clone(), veracity_level);
                Ok(())
            },
        }
    }
}

#[cfg(test)]
pub struct FakeVeracityChecker(pub Vec<CargoPackage>);

#[cfg(test)]
impl VeracityFactorCheck for FakeVeracityChecker {
    async fn execute(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        Ok(self.0.contains(crate_info))
    }
}

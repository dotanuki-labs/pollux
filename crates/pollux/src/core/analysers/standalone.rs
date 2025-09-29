// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::interfaces::{AnalyzedDataStorage, VeracityFactorCheck};
use crate::core::models::{CargoPackage, CrateVeracityChecks};
use crate::infra::caching::analysis::AnalysedPackagesCache;
use crate::infra::networking::crates::OfficialCratesRegistryChecker;
use crate::infra::networking::ossrebuild::OssRebuildChecker;
use url::Url;

pub enum CrateProvenanceChecker {
    CratesOfficialRegistry(OfficialCratesRegistryChecker),
    #[cfg(test)]
    FakeRegistry(FakeVeracityChecker),
}

impl VeracityFactorCheck for CrateProvenanceChecker {
    async fn execute(&self, crate_info: &CargoPackage) -> anyhow::Result<Option<Url>> {
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
    async fn execute(&self, crate_info: &CargoPackage) -> anyhow::Result<Option<Url>> {
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
    FakeCache(HashMap<String, CrateVeracityChecks>),
}

impl AnalyzedDataStorage for CachedDataChecker {
    fn retrieve(&self, crate_info: &CargoPackage) -> anyhow::Result<Option<CrateVeracityChecks>> {
        match self {
            CachedDataChecker::FileSystem(delegate) => delegate.retrieve(crate_info),
            #[cfg(test)]
            CachedDataChecker::FakeCache(fakes) => Ok(fakes.get(&crate_info.name).cloned()),
        }
    }

    fn save(&self, crate_info: &CargoPackage, checks: CrateVeracityChecks) -> anyhow::Result<()> {
        match self {
            CachedDataChecker::FileSystem(delegate) => delegate.save(crate_info, checks),
            #[cfg(test)]
            CachedDataChecker::FakeCache(fakes) => {
                fakes.to_owned().insert(crate_info.name.clone(), checks);
                Ok(())
            },
        }
    }
}

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
pub struct FakeVeracityChecker(pub HashMap<CargoPackage, String>);

#[cfg(test)]
impl VeracityFactorCheck for FakeVeracityChecker {
    async fn execute(&self, cargo_package: &CargoPackage) -> anyhow::Result<Option<Url>> {
        let url = self.0.get(cargo_package).map(|url| Url::from_str(url).unwrap());
        Ok(url)
    }
}

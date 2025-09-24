// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::interfaces::{VeracityEvaluationStorage, VeracityFactorEvaluation};
use crate::core::models::{CargoPackage, CrateVeracityLevel};
use crate::infra::caching::filesystem::DirectoryBased;
use crate::infra::networking::crates::registry::OfficialCratesRegistryEvaluator;
use crate::infra::networking::ossrebuild::OssRebuildEvaluator;
#[cfg(test)]
use std::collections::HashMap;

pub enum CrateProvenanceEvaluator {
    CratesOfficialRegistry(OfficialCratesRegistryEvaluator),
    #[cfg(test)]
    FakeRegistry(FakeVeracityEvaluator),
}

impl VeracityFactorEvaluation for CrateProvenanceEvaluator {
    async fn evaluate(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        match self {
            CrateProvenanceEvaluator::CratesOfficialRegistry(delegate) => delegate.evaluate(crate_info).await,
            #[cfg(test)]
            CrateProvenanceEvaluator::FakeRegistry(fake) => fake.evaluate(crate_info).await,
        }
    }
}

pub enum BuildReproducibilityEvaluator {
    GoogleOssRebuild(OssRebuildEvaluator),
    #[cfg(test)]
    FakeRebuilder(FakeVeracityEvaluator),
}

impl VeracityFactorEvaluation for BuildReproducibilityEvaluator {
    async fn evaluate(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        match self {
            BuildReproducibilityEvaluator::GoogleOssRebuild(delegate) => delegate.evaluate(crate_info).await,
            #[cfg(test)]
            BuildReproducibilityEvaluator::FakeRebuilder(fake) => fake.evaluate(crate_info).await,
        }
    }
}

pub enum CachedExecutionEvaluator {
    FileSystem(DirectoryBased),
    #[cfg(test)]
    FakeCache(HashMap<String, CrateVeracityLevel>),
}

impl VeracityEvaluationStorage for CachedExecutionEvaluator {
    fn read(&self, crate_info: &CargoPackage) -> anyhow::Result<CrateVeracityLevel> {
        match self {
            CachedExecutionEvaluator::FileSystem(delegate) => delegate.read(crate_info),
            #[cfg(test)]
            CachedExecutionEvaluator::FakeCache(fakes) => Ok(fakes
                .get(&crate_info.name)
                .cloned()
                .unwrap_or(CrateVeracityLevel::NotAvailable)),
        }
    }

    fn save(&self, crate_info: &CargoPackage, veracity_level: CrateVeracityLevel) -> anyhow::Result<()> {
        match self {
            CachedExecutionEvaluator::FileSystem(delegate) => delegate.save(crate_info, veracity_level),
            #[cfg(test)]
            CachedExecutionEvaluator::FakeCache(fakes) => {
                fakes.to_owned().insert(crate_info.name.clone(), veracity_level);
                Ok(())
            },
        }
    }
}

#[cfg(test)]
pub struct FakeVeracityEvaluator(pub Vec<CargoPackage>);

#[cfg(test)]
impl VeracityFactorEvaluation for FakeVeracityEvaluator {
    async fn evaluate(&self, crate_info: &CargoPackage) -> anyhow::Result<bool> {
        Ok(self.0.contains(crate_info))
    }
}

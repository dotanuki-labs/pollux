// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use packageurl::PackageUrl;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
}

impl TryFrom<String> for CargoPackage {
    type Error = anyhow::Error;

    fn try_from(value: String) -> anyhow::Result<Self> {
        let purl = PackageUrl::from_str(value.as_str())?;
        let name = purl.name();
        let version = purl.version().expect("");
        let cargo_package = CargoPackage::with(name, version);
        Ok(cargo_package)
    }
}

impl CargoPackage {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    pub fn with(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}

impl Display for CargoPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("pkg:cargo/{}@{}", self.name, self.version))
    }
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum VeracityFactor {
    ReproducibleBuilds,
    ProvenanceAttested,
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum CrateVeracityLevel {
    NotAvailable,
    SingleFactor(VeracityFactor),
    TwoFactors,
}

impl CrateVeracityLevel {
    pub fn to_booleans(&self) -> (bool, bool) {
        match self {
            CrateVeracityLevel::NotAvailable => (false, false),
            CrateVeracityLevel::SingleFactor(factor) => match factor {
                VeracityFactor::ReproducibleBuilds => (false, true),
                VeracityFactor::ProvenanceAttested => (true, false),
            },
            CrateVeracityLevel::TwoFactors => (true, true),
        }
    }

    pub fn from_booleans(provenance: bool, rebuilds: bool) -> Self {
        match (provenance, rebuilds) {
            (true, true) => CrateVeracityLevel::TwoFactors,
            (false, true) => CrateVeracityLevel::SingleFactor(VeracityFactor::ReproducibleBuilds),
            (true, false) => CrateVeracityLevel::SingleFactor(VeracityFactor::ProvenanceAttested),
            (false, false) => CrateVeracityLevel::NotAvailable,
        }
    }
}

pub type EvaluationOutcome = (CargoPackage, Option<CrateVeracityLevel>);

pub struct PolluxStatistics {
    pub total_project_packages: usize,
    pub with_veracity_level: usize,
    pub without_veracity_level: usize,
}

pub struct PolluxResults {
    pub statistics: PolluxStatistics,
    pub outcomes: Vec<EvaluationOutcome>,
}

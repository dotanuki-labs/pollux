// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use packageurl::PackageUrl;
use std::fmt::{Display, Formatter};
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

impl Display for CrateVeracityLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CrateVeracityLevel::NotAvailable => f.write_str("none"),
            CrateVeracityLevel::SingleFactor(factor) => match factor {
                VeracityFactor::ReproducibleBuilds => f.write_str("reproducible builds"),
                VeracityFactor::ProvenanceAttested => f.write_str("provenance attested"),
            },
            CrateVeracityLevel::TwoFactors => f.write_str("provenance attested; reproducible builds"),
        }
    }
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

pub struct StatisticsForPackages {
    pub total: usize,
    pub provenance_attested: usize,
    pub reproducible_builds: usize,
}

pub struct PolluxResults {
    pub statistics: StatisticsForPackages,
    pub outcomes: Vec<EvaluationOutcome>,
}

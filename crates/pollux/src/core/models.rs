// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use clap::ValueEnum;
use packageurl::PackageUrl;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use url::Url;

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
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
pub struct CrateVeracityChecks {
    pub provenance_evidence: Option<Url>,
    pub reproducibility_evidence: Option<Url>,
}

impl CrateVeracityChecks {
    pub fn new(provenance_evidence: Option<Url>, reproducibility_evidence: Option<Url>) -> Self {
        Self {
            provenance_evidence,
            reproducibility_evidence,
        }
    }
}

impl Display for CrateVeracityChecks {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (&self.provenance_evidence, &self.reproducibility_evidence) {
            (Some(_), Some(_)) => f.write_str("provenance attested; reproducible builds"),
            (Some(_), None) => f.write_str("provenance attested"),
            (None, Some(_)) => f.write_str("reproducible builds"),
            (None, None) => f.write_str("none"),
        }
    }
}

#[derive(ValueEnum, Debug, Clone)]
pub enum CleanupScope {
    Everything,
    AnalysedData,
    PackageSources,
}

pub type AnalysisOutcome = (CargoPackage, Option<CrateVeracityChecks>);
pub type InquiringOutcome = (CargoPackage, CrateVeracityChecks);

pub struct StatisticsForPackages {
    pub total: usize,
    pub provenance_attested: usize,
    pub reproducible_builds: usize,
}

pub struct AnalysisResults {
    pub statistics: StatisticsForPackages,
    pub outcomes: Vec<AnalysisOutcome>,
}

pub struct EcosystemInquiringResults {
    pub percentual_presence_of_provance: f32,
    pub percentual_presence_of_reproducibility: f32,
    pub outcomes: Vec<InquiringOutcome>,
}

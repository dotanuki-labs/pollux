// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use clap::ValueEnum;
use packageurl::PackageUrl;
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use url::Url;

#[derive(Clone, Debug, PartialEq, Hash, Eq, Serialize)]
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("pkg:cargo/{}@{}", self.name, self.version))
    }
}

#[derive(Clone, Debug, PartialEq, Hash, Eq, Serialize)]
pub struct CrateVeracityChecks {
    pub trusted_publishing_evidence: Option<Url>,
    pub reproducibility_evidence: Option<Url>,
}

impl CrateVeracityChecks {
    pub fn new(trusted_publishing_evidence: Option<Url>, reproducibility_evidence: Option<Url>) -> Self {
        Self {
            trusted_publishing_evidence,
            reproducibility_evidence,
        }
    }
}

impl Display for CrateVeracityChecks {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (&self.trusted_publishing_evidence, &self.reproducibility_evidence) {
            (Some(_), Some(_)) => f.write_str("trusted publishing; reproducible builds"),
            (Some(_), None) => f.write_str("trusted publishing"),
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

#[derive(ValueEnum, Debug, Clone)]
pub enum InquireReportKind {
    Console,
    Html,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum InquireCoverage {
    Small,
    Medium,
    Large,
    Huge,
}

pub type AnalysisOutcome = (CargoPackage, Option<CrateVeracityChecks>);

pub struct StatisticsForPackages {
    pub total: usize,
    pub trusted_publishing: usize,
    pub reproducible_builds: usize,
}

pub struct AnalysisResults {
    pub statistics: StatisticsForPackages,
    pub outcomes: Vec<AnalysisOutcome>,
}

#[derive(Serialize, Debug)]
pub struct InquiringOutcome {
    pub cargo_package: CargoPackage,
    pub checks: CrateVeracityChecks,
}

#[derive(Serialize, Debug)]
pub struct EcosystemInquiringResults {
    pub total_crates_inquired: u32,
    pub total_crates_with_trusted_publishing: u32,
    pub total_crates_with_reproducibility: u32,
    pub presence_of_trusted_publishing: String,
    pub presence_of_reproducibility: String,
    pub outcomes: Vec<InquiringOutcome>,
}

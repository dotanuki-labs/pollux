// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::{CargoPackage, CrateVeracityLevel};
use url::Url;

pub mod check;
pub mod evaluation;

pub type EvaluationOutcome = (CargoPackage, Option<CrateVeracityLevel>);

pub struct CrateChecks {
    pub provenance_evidence: Option<Url>,
    pub reproducibility_evidence: Option<Url>,
}

impl CrateChecks {
    pub fn new(provenance_evidence: Option<Url>, reproducibility_evidence: Option<Url>) -> Self {
        Self {
            provenance_evidence,
            reproducibility_evidence,
        }
    }
}

pub struct StatisticsForPackages {
    pub total: usize,
    pub provenance_attested: usize,
    pub reproducible_builds: usize,
}

pub struct EvaluationResults {
    pub statistics: StatisticsForPackages,
    pub outcomes: Vec<EvaluationOutcome>,
}

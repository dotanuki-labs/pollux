// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::infra::{ReproducibleBuildsEvaluator, TrustedPublishingEvaluator};
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
}

impl CrateInfo {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }
}

impl Display for CrateInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("pkg:cargo/{}@{}", self.name, self.version))
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum VerificationKind {
    ReproducedBuild,
    UsesTrustedPublishing,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum TruthfulnessVerification {
    NotAvailable,
    Partial(VerificationKind),
    Total,
}

#[allow(dead_code)]
pub trait TruthfulnessEvaluation {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<bool>;
}

pub struct TruthfulnessEvaluator {
    trusted_publishing_evaluator: TrustedPublishingEvaluator,
    reproducible_builds_evaluator: ReproducibleBuildsEvaluator,
}

#[allow(dead_code)]
impl TruthfulnessEvaluator {
    pub fn new(
        trusted_publishing_evaluator: TrustedPublishingEvaluator,
        reproducible_builds_evaluator: ReproducibleBuildsEvaluator,
    ) -> Self {
        Self {
            trusted_publishing_evaluator,
            reproducible_builds_evaluator,
        }
    }

    pub async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<TruthfulnessVerification> {
        let uses_trusted_publishing = self.trusted_publishing_evaluator.evaluate(crate_info).await?;
        let has_reproduced_build = self.reproducible_builds_evaluator.evaluate(crate_info).await?;

        let verification = match (uses_trusted_publishing, has_reproduced_build) {
            (true, true) => TruthfulnessVerification::Total,
            (false, true) => TruthfulnessVerification::Partial(VerificationKind::ReproducedBuild),
            (true, false) => TruthfulnessVerification::Partial(VerificationKind::UsesTrustedPublishing),
            (false, false) => TruthfulnessVerification::NotAvailable,
        };

        Ok(verification)
    }
}

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::infra::{ReproducibleBuildsEvaluator, TrustedPublishingEvaluator};

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
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

#[allow(dead_code)]
pub struct TruthfulnessEvaluator {
    trusted_publishing_evaluator: TrustedPublishingEvaluator,
    reproducible_builds_evaluator: ReproducibleBuildsEvaluator,
}

#[allow(dead_code)]
impl TruthfulnessEvaluator {
    async fn evaluate(&self, crate_info: &CrateInfo) -> anyhow::Result<TruthfulnessVerification> {
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

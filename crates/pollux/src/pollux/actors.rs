// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::evaluators::combined::CombinedVeracityEvaluator;
use crate::core::interfaces::CrateVeracityEvaluation;
use crate::core::models::{CrateVeracityLevel, EvaluationOutcome, PolluxResults, PolluxStatistics};
use crate::pollux::PolluxMessage;
use ractor::{Actor, ActorProcessingErr, ActorRef};

pub struct PolluxEvaluatorActor {
    veracity_evaluator: CombinedVeracityEvaluator,
}

impl PolluxEvaluatorActor {
    pub fn new(veracity_evaluator: CombinedVeracityEvaluator) -> Self {
        Self { veracity_evaluator }
    }
}

impl Actor for PolluxEvaluatorActor {
    type Msg = PolluxMessage;
    type State = Vec<EvaluationOutcome>;
    type Arguments = u64;

    async fn pre_start(&self, _: ActorRef<Self::Msg>, _: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        Ok(vec![])
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        packages: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PolluxMessage::EvaluatePackage(cargo_package) => {
                log::info!("[pollux.actor] starting evaluation for package {}", &cargo_package);
                let maybe_evaluated = self.veracity_evaluator.evaluate(&cargo_package).await.ok();
                log::info!("[pollux.actor] finished evaluation for package {}", &cargo_package);
                packages.push((cargo_package, maybe_evaluated));
            },
            PolluxMessage::AggregateResults(reply) => {
                log::info!("[pollux.actor] computing aggregated results for processed packages");

                let total_evaluated_packages = packages
                    .iter()
                    .filter(|(_, evaluation)| evaluation.is_some())
                    .collect::<Vec<_>>()
                    .len();

                let total_packages_with_veracity_level = packages
                    .iter()
                    .filter_map(|(_, evaluation)| evaluation.as_ref())
                    .filter(|veracity_level| **veracity_level != CrateVeracityLevel::NotAvailable)
                    .collect::<Vec<_>>()
                    .len();

                let statistics = PolluxStatistics {
                    total_project_packages: total_evaluated_packages,
                    with_veracity_level: total_packages_with_veracity_level,
                    without_veracity_level: total_evaluated_packages - total_packages_with_veracity_level,
                };

                let results = PolluxResults {
                    statistics,
                    outcomes: packages.clone(),
                };

                if reply.send(results).is_err() {
                    log::error!("[pollux.actor] cannot reply with state");
                }
            },
        }

        Ok(())
    }
}

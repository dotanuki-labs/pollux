// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::{CargoPackage, CombinedVeracityEvaluator, CrateVeracityEvaluation, CrateVeracityLevel};
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};

pub type EvaluationOutcome = (CargoPackage, Option<CrateVeracityLevel>);

pub struct PolluxStatistics {
    pub with_veracity_level: usize,
    pub without_veracity_level: usize,
}

pub struct PolluxResults {
    pub statistics: PolluxStatistics,
    pub outcomes: Vec<EvaluationOutcome>,
}

pub enum PolluxMessage {
    Evaluate(CargoPackage),
    AggregateResults(RpcReplyPort<PolluxResults>),
}

pub struct Pollux {
    evaluator: CombinedVeracityEvaluator,
}

impl Pollux {
    pub fn new(evaluator: CombinedVeracityEvaluator) -> Self {
        Self { evaluator }
    }
}

impl Actor for Pollux {
    type Msg = PolluxMessage;
    type State = Vec<EvaluationOutcome>;
    type Arguments = ();

    async fn pre_start(&self, _: ActorRef<Self::Msg>, _: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        Ok(vec![])
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PolluxMessage::Evaluate(cargo_package) => {
                log::info!("[pollux.actor] starting evaluation for package {:?}", &cargo_package);
                let maybe_evaluated = self.evaluator.evaluate(&cargo_package).await.ok();
                log::info!("[pollux.actor] finished evaluation for package {:?}", &cargo_package);
                state.push((cargo_package, maybe_evaluated));
            },
            PolluxMessage::AggregateResults(reply) => {
                let total_evaluated_packages = state
                    .iter()
                    .filter(|(_, evaluation)| evaluation.is_some())
                    .collect::<Vec<_>>()
                    .len();

                let total_packages_with_veracity_level = state
                    .iter()
                    .filter_map(|(_, evaluation)| evaluation.as_ref())
                    .filter(|veracity_level| **veracity_level != CrateVeracityLevel::NotAvailable)
                    .collect::<Vec<_>>()
                    .len();

                let statistics = PolluxStatistics {
                    with_veracity_level: total_packages_with_veracity_level,
                    without_veracity_level: total_evaluated_packages - total_packages_with_veracity_level,
                };

                let results = PolluxResults {
                    statistics,
                    outcomes: state.clone(),
                };

                if reply.send(results).is_err() {
                    log::info!("[pollux.actor] cannot reply with state");
                }
            },
        }

        Ok(())
    }
}

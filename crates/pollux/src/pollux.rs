// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::{CargoPackage, CombinedVeracityEvaluator, CrateVeracityEvaluation, CrateVeracityLevel};
use crate::infra::cargo::RustProjectDependenciesResolver;
use crate::ioc::CRATESIO_MILLIS_TO_WAIT_AFTER_RATE_LIMITED;
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use std::path::PathBuf;

pub enum PolluxTask {
    EvaluateRustProject(PathBuf),
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

pub enum PolluxMessage {
    EvaluatePackage(CargoPackage),
    AggregateResults(RpcReplyPort<PolluxResults>),
}

pub struct Pollux {
    dependencies_resolver: RustProjectDependenciesResolver,
    pollux_executor: PolluxExecutor,
}

impl Pollux {
    pub fn new(dependencies_resolver: RustProjectDependenciesResolver, pollux_executor: PolluxExecutor) -> Self {
        Self {
            dependencies_resolver,
            pollux_executor,
        }
    }

    pub async fn execute(self) -> anyhow::Result<PolluxResults> {
        let cargo_packages = self.dependencies_resolver.resolve_packages()?;
        let total_project_packages = cargo_packages.len();

        let (actor, _) = Actor::spawn(None, self.pollux_executor, ()).await?;
        for package in cargo_packages {
            actor.cast(PolluxMessage::EvaluatePackage(package))?
        }

        let max_timeout = CRATESIO_MILLIS_TO_WAIT_AFTER_RATE_LIMITED * 2 * total_project_packages as u64;
        let results = ractor::call_t!(actor, PolluxMessage::AggregateResults, max_timeout)?;
        Ok(results)
    }
}

pub struct PolluxExecutor {
    veracity_evaluator: CombinedVeracityEvaluator,
}

impl PolluxExecutor {
    pub fn new(veracity_evaluator: CombinedVeracityEvaluator) -> Self {
        Self { veracity_evaluator }
    }
}

impl Actor for PolluxExecutor {
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
            PolluxMessage::EvaluatePackage(cargo_package) => {
                log::info!("[pollux.actor] starting evaluation for package {}", &cargo_package);
                let maybe_evaluated = self.veracity_evaluator.evaluate(&cargo_package).await.ok();
                log::info!("[pollux.actor] finished evaluation for package {}", &cargo_package);
                state.push((cargo_package, maybe_evaluated));
            },
            PolluxMessage::AggregateResults(reply) => {
                log::info!("[pollux.actor] computing aggregated results for processed packages");

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
                    total_project_packages: total_evaluated_packages,
                    with_veracity_level: total_packages_with_veracity_level,
                    without_veracity_level: total_evaluated_packages - total_packages_with_veracity_level,
                };

                let results = PolluxResults {
                    statistics,
                    outcomes: state.clone(),
                };

                if reply.send(results).is_err() {
                    log::error!("[pollux.actor] cannot reply with state");
                }
            },
        }

        Ok(())
    }
}

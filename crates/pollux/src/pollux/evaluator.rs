// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::evaluators::combined::CombinedVeracityEvaluator;
use crate::core::interfaces::CrateVeracityLevelEvaluation;
use crate::core::models::{CargoPackage, CrateVeracityLevel, VeracityFactor};
use crate::infra::networking::crates::resolvers::DependenciesResolver;
use crate::ioc::MILLIS_TO_WAIT_AFTER_RATE_LIMITED;
use console::style;
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use std::path::Path;

pub type EvaluationOutcome = (CargoPackage, Option<CrateVeracityLevel>);

pub struct StatisticsForPackages {
    pub total: usize,
    pub provenance_attested: usize,
    pub reproducible_builds: usize,
}

pub struct EvaluationResults {
    pub statistics: StatisticsForPackages,
    pub outcomes: Vec<EvaluationOutcome>,
}

pub enum EvaluationMessage {
    EvaluatePackage(CargoPackage),
    AggregateResults(RpcReplyPort<EvaluationResults>),
}

pub struct PolluxEvaluatorActor {
    dependencies_resolver: DependenciesResolver,
    veracity_evaluator: CombinedVeracityEvaluator,
}

impl PolluxEvaluatorActor {
    pub fn new(dependencies_resolver: DependenciesResolver, veracity_evaluator: CombinedVeracityEvaluator) -> Self {
        Self {
            dependencies_resolver,
            veracity_evaluator,
        }
    }

    pub async fn evaluate_local_project(self, project_path: &Path) -> anyhow::Result<()> {
        self.show_evaluation_disclaimer();
        let cargo_packages = self
            .dependencies_resolver
            .resolve_for_local_project(project_path)
            .await?;
        self.evaluate_packages(cargo_packages).await
    }

    pub async fn evaluate_crate_package(self, cargo_package: &CargoPackage) -> anyhow::Result<()> {
        self.show_evaluation_disclaimer();
        let cargo_packages = self
            .dependencies_resolver
            .resolve_for_crate_package(cargo_package)
            .await?;
        self.evaluate_packages(cargo_packages).await
    }

    async fn evaluate_packages(self, cargo_packages: Vec<CargoPackage>) -> anyhow::Result<()> {
        let total_project_packages = cargo_packages.len() as u64;
        let (actor, _) = Actor::spawn(None, self, total_project_packages).await?;

        for package in cargo_packages {
            actor.cast(EvaluationMessage::EvaluatePackage(package))?
        }

        let max_timeout = MILLIS_TO_WAIT_AFTER_RATE_LIMITED * 2 * total_project_packages;
        let results = ractor::call_t!(actor, EvaluationMessage::AggregateResults, max_timeout)?;
        Self::show_evaluation_results(&results);
        Ok(())
    }

    fn show_evaluation_disclaimer(&self) {
        println!();
        println!("Evaluating veracity for packages. This operation may take some time ...");
    }

    fn show_evaluation_results(results: &EvaluationResults) {
        let statistics = &results.statistics;
        println!();
        println!("Statistics : ");
        println!();
        println!("• total packages evaluated : {}", statistics.total);
        println!("• with provenance attested : {}", statistics.provenance_attested);
        println!("• with reproducible builds : {}", statistics.reproducible_builds);
        println!();
        println!("Veracity factors : ");
        println!();
        results
            .outcomes
            .iter()
            .for_each(|(package, maybe_veracity_check)| match maybe_veracity_check {
                Some(level) => {
                    println!("• {} ({}) ", package, style(level).cyan());
                },
                None => {
                    println!("• {} : {}", package, style("failed to evaluate").red());
                },
            });

        println!();
    }
}

impl Actor for PolluxEvaluatorActor {
    type Msg = EvaluationMessage;
    type State = Vec<EvaluationOutcome>;
    type Arguments = u64;

    async fn pre_start(&self, _: ActorRef<Self::Msg>, _: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        Ok(vec![])
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        outcomes: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            EvaluationMessage::EvaluatePackage(cargo_package) => {
                log::info!("[pollux.actor] starting evaluation for package {}", &cargo_package);
                let maybe_evaluated = self.veracity_evaluator.evaluate(&cargo_package).await.ok();
                log::info!("[pollux.actor] finished evaluation for package {}", &cargo_package);
                outcomes.push((cargo_package, maybe_evaluated));
            },
            EvaluationMessage::AggregateResults(reply) => {
                log::info!("[pollux.actor] computing aggregated results for processed packages");

                let mut total_evaluated_packages = 0;
                let mut with_provenance = 0;
                let mut with_reproducible_builds = 0;

                for (package, evaluation) in outcomes.iter() {
                    total_evaluated_packages += 1;

                    if let Some(level) = evaluation {
                        match level {
                            CrateVeracityLevel::NotAvailable => {
                                log::info!("[pollux.actor] no stats for : {}", &package);
                            },
                            CrateVeracityLevel::SingleFactor(factor) => match factor {
                                VeracityFactor::ReproducibleBuilds => with_reproducible_builds += 1,
                                VeracityFactor::ProvenanceAttested => with_provenance += 1,
                            },
                            CrateVeracityLevel::TwoFactors => {
                                with_reproducible_builds += 1;
                                with_provenance += 1
                            },
                        }
                    }
                }

                let statistics = StatisticsForPackages {
                    total: total_evaluated_packages,
                    provenance_attested: with_provenance,
                    reproducible_builds: with_reproducible_builds,
                };

                let results = EvaluationResults {
                    statistics,
                    outcomes: outcomes.clone(),
                };

                if reply.send(results).is_err() {
                    log::error!("[pollux.actor] cannot reply with state");
                }
            },
        }

        Ok(())
    }
}

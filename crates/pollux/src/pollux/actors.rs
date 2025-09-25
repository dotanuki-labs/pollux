// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::evaluators::combined::CombinedVeracityEvaluator;
use crate::core::interfaces::CrateVeracityLevelEvaluation;
use crate::core::models::{
    CrateVeracityLevel, EvaluationOutcome, PolluxResults, StatisticsForPackages, VeracityFactor,
};
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
        outcomes: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PolluxMessage::EvaluatePackage(cargo_package) => {
                log::info!("[pollux.actor] starting evaluation for package {}", &cargo_package);
                let maybe_evaluated = self.veracity_evaluator.evaluate(&cargo_package).await.ok();
                log::info!("[pollux.actor] finished evaluation for package {}", &cargo_package);
                outcomes.push((cargo_package, maybe_evaluated));
            },
            PolluxMessage::AggregateResults(reply) => {
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

                let results = PolluxResults {
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

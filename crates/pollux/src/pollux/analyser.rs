// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::analysers::combined::VeracityChecksAnalyser;
use crate::core::interfaces::CrateVeracityAnalysis;
use crate::core::models::{AnalysisOutcome, AnalysisResults, CargoPackage, StatisticsForPackages};
use crate::infra::networking::crates::resolvers::DependenciesResolver;
use crate::ioc::MILLIS_TO_WAIT_AFTER_RATE_LIMITED;
use camino::Utf8PathBuf;
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};

pub enum AnalyserMessage {
    AnalysePackage(CargoPackage),
    AggregateResults(RpcReplyPort<AnalysisResults>),
}

pub struct PolluxAnalyser {
    dependencies_resolver: DependenciesResolver,
    veracity_analyser: VeracityChecksAnalyser,
}

impl PolluxAnalyser {
    pub fn new(dependencies_resolver: DependenciesResolver, veracity_analyser: VeracityChecksAnalyser) -> Self {
        Self {
            dependencies_resolver,
            veracity_analyser,
        }
    }

    pub async fn analyse_project(self, project_path: Utf8PathBuf) -> anyhow::Result<AnalysisResults> {
        let cargo_packages = self
            .dependencies_resolver
            .resolve_for_local_project(project_path)
            .await?;
        self.analyse_packages(cargo_packages).await
    }

    pub async fn analyse_package(self, cargo_package: &CargoPackage) -> anyhow::Result<AnalysisResults> {
        let cargo_packages = self
            .dependencies_resolver
            .resolve_for_crate_package(cargo_package)
            .await?;
        self.analyse_packages(cargo_packages).await
    }

    async fn analyse_packages(self, cargo_packages: Vec<CargoPackage>) -> anyhow::Result<AnalysisResults> {
        let total_project_packages = cargo_packages.len() as u64;
        let (actor, _) = Actor::spawn(None, self, total_project_packages).await?;

        for package in cargo_packages {
            actor.cast(AnalyserMessage::AnalysePackage(package))?
        }

        let max_timeout = MILLIS_TO_WAIT_AFTER_RATE_LIMITED * 2 * total_project_packages;
        let results = ractor::call_t!(actor, AnalyserMessage::AggregateResults, max_timeout)?;
        Ok(results)
    }
}

impl Actor for PolluxAnalyser {
    type Msg = AnalyserMessage;
    type State = Vec<AnalysisOutcome>;
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
            AnalyserMessage::AnalysePackage(cargo_package) => {
                log::info!("[pollux.actor] starting analysis for package {}", &cargo_package);
                let maybe_analysed = self.veracity_analyser.execute(&cargo_package).await.ok();
                log::info!("[pollux.actor] finished analysis for package {}", &cargo_package);
                outcomes.push((cargo_package, maybe_analysed));
            },
            AnalyserMessage::AggregateResults(reply) => {
                log::info!("[pollux.actor] computing aggregated results for processed packages");

                let mut total_analysed_packages = 0;
                let mut with_trusted_publishing = 0;
                let mut with_reproducible_builds = 0;

                for (package, checks) in outcomes.iter() {
                    total_analysed_packages += 1;

                    if let Some(existing) = checks {
                        match (
                            &existing.trusted_publishing_evidence,
                            &existing.reproducibility_evidence,
                        ) {
                            (Some(_), Some(_)) => {
                                with_reproducible_builds += 1;
                                with_trusted_publishing += 1
                            },

                            (Some(_), None) => with_trusted_publishing += 1,

                            (None, Some(_)) => with_reproducible_builds += 1,
                            (None, None) => {
                                log::info!("[pollux.actor] no stats for : {}", &package);
                            },
                        }
                    }
                }

                let statistics = StatisticsForPackages {
                    total: total_analysed_packages,
                    trusted_publishing: with_trusted_publishing,
                    reproducible_builds: with_reproducible_builds,
                };

                let results = AnalysisResults {
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

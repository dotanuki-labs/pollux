// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod actors;

use crate::core::models::{CargoPackage, PolluxResults};
use crate::infra::caching::CacheManager;
use crate::infra::networking::crates::resolvers::DependenciesResolver;
use crate::ioc::MILLIS_TO_WAIT_AFTER_RATE_LIMITED;
use crate::pollux::actors::PolluxEvaluatorActor;
use console::style;
use ractor::{Actor, RpcReplyPort};
use std::path::{Path, PathBuf};

pub enum PolluxTask {
    EvaluateRustProject(PathBuf),
    EvaluateRustCrate(CargoPackage),
    CleanupEverything,
    CleanupPackages,
    CleanupEvaluations,
}

pub enum PolluxMessage {
    EvaluatePackage(CargoPackage),
    AggregateResults(RpcReplyPort<PolluxResults>),
}

pub struct Pollux {
    cache_manager: CacheManager,
    dependencies_resolver: DependenciesResolver,
    pollux_actor_factory: fn() -> PolluxEvaluatorActor,
}

impl Pollux {
    pub fn new(
        cache_manager: CacheManager,
        dependencies_resolver: DependenciesResolver,
        pollux_evaluator_factory: fn() -> PolluxEvaluatorActor,
    ) -> Self {
        Self {
            cache_manager,
            dependencies_resolver,
            pollux_actor_factory: pollux_evaluator_factory,
        }
    }

    pub async fn evaluate_local_project(&self, project_path: &Path) -> anyhow::Result<()> {
        self.show_evaluation_disclaimer();
        let cargo_packages = self
            .dependencies_resolver
            .resolve_for_local_project(project_path)
            .await?;
        self.evaluate_packages(cargo_packages).await
    }

    pub async fn evaluate_crate_package(&self, cargo_package: &CargoPackage) -> anyhow::Result<()> {
        self.show_evaluation_disclaimer();
        let cargo_packages = self
            .dependencies_resolver
            .resolve_for_crate_package(cargo_package)
            .await?;
        self.evaluate_packages(cargo_packages).await
    }

    pub fn cleanup_cached_evaluations(&self) {
        self.cache_manager.cleanup_evaluations();
        println!("Cached evaluations removed with success!");
    }

    pub fn cleanup_cached_packages(&self) {
        self.cache_manager.cleanup_packages();
        println!("Cached packages removed with success!");
    }

    pub fn cleanup_everything(&self) {
        self.cache_manager.cleanup_all();
        println!("All caches removed with success!");
    }

    async fn evaluate_packages(&self, cargo_packages: Vec<CargoPackage>) -> anyhow::Result<()> {
        let total_project_packages = cargo_packages.len() as u64;
        let evaluator_factory = self.pollux_actor_factory;
        let (actor, _) = Actor::spawn(None, evaluator_factory(), total_project_packages).await?;
        for package in cargo_packages {
            actor.cast(PolluxMessage::EvaluatePackage(package))?
        }

        let max_timeout = MILLIS_TO_WAIT_AFTER_RATE_LIMITED * 2 * total_project_packages;
        let results = ractor::call_t!(actor, PolluxMessage::AggregateResults, max_timeout)?;
        self.show_evaluation_results(&results);
        Ok(())
    }

    fn show_evaluation_disclaimer(&self) {
        println!();
        println!("Evaluating veracity for packages. This operation may take some time ...");
    }

    fn show_evaluation_results(&self, results: &PolluxResults) {
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

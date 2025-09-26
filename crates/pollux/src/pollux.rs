// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod actors;

use crate::core::models::CargoPackage;
use crate::infra::caching::CacheManager;
use crate::infra::networking::crates::resolvers::DependenciesResolver;
use crate::ioc::MILLIS_TO_WAIT_AFTER_RATE_LIMITED;
use crate::pollux::actors::EvaluationResults;
use crate::pollux::actors::check::PolluxStandalonePackageChecker;
use crate::pollux::actors::evaluation::{PolluxEvaluationMessage, PolluxEvaluatorActor};
use console::style;
use ractor::Actor;
use std::path::{Path, PathBuf};

pub enum PolluxTask {
    CheckRustCrate(CargoPackage),
    CleanupEverything,
    CleanupPackages,
    CleanupEvaluations,
    EvaluateRustProject(PathBuf),
    EvaluateRustCrate(CargoPackage),
}

pub struct Pollux {
    cache_manager: CacheManager,
    dependencies_resolver: DependenciesResolver,
    pollux_evaluator_factory: fn() -> PolluxEvaluatorActor,
    standalone_package_checker: PolluxStandalonePackageChecker,
}

impl Pollux {
    pub fn new(
        cache_manager: CacheManager,
        dependencies_resolver: DependenciesResolver,
        pollux_evaluator_factory: fn() -> PolluxEvaluatorActor,
        standalone_package_checker: PolluxStandalonePackageChecker,
    ) -> Self {
        Self {
            cache_manager,
            dependencies_resolver,
            pollux_evaluator_factory,
            standalone_package_checker,
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

    pub async fn check_crate_package(&self, cargo_package: &CargoPackage) -> anyhow::Result<()> {
        println!();
        println!("Checking veracity factors for : {}", cargo_package);
        println!();

        let checks = self.standalone_package_checker.check(cargo_package).await?;

        if let Some(cratesio_link) = checks.provenance_evidence {
            println!(
                "• provenance evidence (v{} via github): {}",
                cargo_package.version,
                style(cratesio_link).cyan()
            );
        } else {
            println!("• provenance evidence : not found");
        }

        if let Some(oss_rebuild_link) = checks.reproducibility_evidence {
            println!("• reproducibility evidence : {}", style(oss_rebuild_link).cyan());
        } else {
            println!("• reproducibility evidence : not found");
        }

        println!();
        Ok(())
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
        let evaluator_factory = self.pollux_evaluator_factory;
        let (actor, _) = Actor::spawn(None, evaluator_factory(), total_project_packages).await?;

        for package in cargo_packages {
            actor.cast(PolluxEvaluationMessage::EvaluatePackage(package))?
        }

        let max_timeout = MILLIS_TO_WAIT_AFTER_RATE_LIMITED * 2 * total_project_packages;
        let results = ractor::call_t!(actor, PolluxEvaluationMessage::AggregateResults, max_timeout)?;
        self.show_evaluation_results(&results);
        Ok(())
    }

    fn show_evaluation_disclaimer(&self) {
        println!();
        println!("Evaluating veracity for packages. This operation may take some time ...");
    }

    fn show_evaluation_results(&self, results: &EvaluationResults) {
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

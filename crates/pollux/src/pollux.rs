// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod actors;

use crate::core::interfaces::PackagesResolution;
use crate::core::models::{CargoPackage, PolluxResults};
use crate::infra::networking::crates::cargo::DependenciesResolver;
use crate::ioc::MILLIS_TO_WAIT_AFTER_RATE_LIMITED;
use crate::pollux::actors::PolluxEvaluatorActor;
use ractor::{Actor, RpcReplyPort};
use std::path::PathBuf;

pub enum PolluxTask {
    EvaluateRustProject(PathBuf),
    EvaluateRustCrate(CargoPackage),
}

pub enum PolluxMessage {
    EvaluatePackage(CargoPackage),
    AggregateResults(RpcReplyPort<PolluxResults>),
}

pub struct Pollux {
    dependencies_resolver: DependenciesResolver,
    pollux_actor: PolluxEvaluatorActor,
}

impl Pollux {
    pub fn new(dependencies_resolver: DependenciesResolver, pollux_actor: PolluxEvaluatorActor) -> Self {
        Self {
            dependencies_resolver,
            pollux_actor,
        }
    }

    pub async fn execute(self) -> anyhow::Result<PolluxResults> {
        let cargo_packages = self.dependencies_resolver.resolve().await?;
        let total_project_packages = cargo_packages.len() as u64;

        let (actor, _) = Actor::spawn(None, self.pollux_actor, total_project_packages).await?;
        for package in cargo_packages {
            actor.cast(PolluxMessage::EvaluatePackage(package))?
        }

        let max_timeout = MILLIS_TO_WAIT_AFTER_RATE_LIMITED * 2 * total_project_packages;
        let results = ractor::call_t!(actor, PolluxMessage::AggregateResults, max_timeout)?;
        Ok(results)
    }
}

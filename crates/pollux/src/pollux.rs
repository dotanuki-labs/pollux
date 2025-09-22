// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub mod actors;

use crate::core::models::{CargoPackage, PolluxResults};
use crate::infra::networking::crates::cargo::{DependenciesResolver, PackagesResolution};
use crate::ioc::CRATESIO_MILLIS_TO_WAIT_AFTER_RATE_LIMITED;
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
    pollux_executor: PolluxEvaluatorActor,
}

impl Pollux {
    pub fn new(dependencies_resolver: DependenciesResolver, pollux_executor: PolluxEvaluatorActor) -> Self {
        Self {
            dependencies_resolver,
            pollux_executor,
        }
    }

    pub async fn execute(self) -> anyhow::Result<PolluxResults> {
        let cargo_packages = self.dependencies_resolver.resolve().await?;
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

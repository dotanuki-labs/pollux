// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

mod core;
mod infra;
mod ioc;
mod pollux;

use crate::infra::cli;
use crate::pollux::PolluxTask;
use PolluxTask::*;
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::troubleshooting::setup_troubleshooting();
    let pollux = ioc::create_pollux();

    let task = cli::parsing::parse_arguments()?;

    match task {
        EvaluateRustProject(project_root) => pollux.evaluate_local_project(project_root.as_path()).await?,
        EvaluateRustCrate(cargo_package) => pollux.evaluate_crate_package(&cargo_package).await?,
        CleanupEverything => {
            pollux.cleanup_cached_evaluations()?;
            pollux.cleanup_cached_packages()?;
        },
        CleanupPackages => pollux.cleanup_cached_packages()?,
        CleanupEvaluations => pollux.cleanup_cached_evaluations()?,
    }

    Ok(())
}

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

mod core;
mod infra;
mod pollux;

use crate::infra::cargo::RustProjectDependenciesResolver;
use crate::pollux::{Pollux, PolluxMessage};
use clap::Parser;
use console::style;
use ractor::Actor;
use std::path::PathBuf;
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ProgramArguments {
    #[arg(short, long, help = "Path pointing to project root")]
    pub path: PathBuf,
}

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    better_panic::install();
    human_panic::setup_panic!();

    env_logger::builder()
        .format_timestamp(None)
        .format_module_path(false)
        .format_level(false)
        .format_file(false)
        .format_target(false)
        .init();

    let arguments = ProgramArguments::parse();

    let veracity_evaluator = core::factory::create_veracity_evaluator(
        infra::factories::cached_evaluator,
        infra::factories::provenance_evaluator,
        infra::factories::reproducibility_evaluator,
    );

    let dependencies_resolver = RustProjectDependenciesResolver::new(arguments.path);

    let cargo_packages = dependencies_resolver.resolve_packages()?;
    let total_project_packages = cargo_packages.len();

    println!("Evaluating veracity for packages. This operation may take some time ...");

    let pollux = Pollux::new(veracity_evaluator);

    let (actor, _) = Actor::spawn(None, pollux, ()).await?;
    for package in cargo_packages {
        actor.cast(PolluxMessage::Evaluate(package))?
    }

    let timeout = 1100 * 2 * total_project_packages as u64;
    let results = ractor::call_t!(actor, PolluxMessage::AggregateResults, timeout)?;

    let statistics = results.statistics;

    println!();
    println!("Packages evaluated : {}", total_project_packages);
    println!("Missing veracity factors : {}", statistics.without_veracity_level);
    println!("With existing factors : {}", statistics.with_veracity_level);
    println!();

    results
        .outcomes
        .iter()
        .for_each(|(package, maybe_veracity_check)| match maybe_veracity_check {
            Some(level) => {
                println!("For {} : veracity = {:?} ", package, style(level).cyan());
            },
            None => {
                println!("For {} : {}", package, style("failed to evaluate").red());
            },
        });

    println!();
    Ok(())
}

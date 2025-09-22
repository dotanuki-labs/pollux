// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

mod core;
mod infra;
mod ioc;
mod pollux;

use crate::infra::cli;
use console::style;
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

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

    let task = cli::parse_arguments()?;
    let pollux = ioc::create_pollux(task);

    println!("Evaluating veracity for packages. This operation may take some time ...");

    let results = pollux.execute().await?;
    let statistics = results.statistics;

    println!();
    println!("Packages evaluated : {}", statistics.total_project_packages);
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

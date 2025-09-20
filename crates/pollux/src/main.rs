// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

mod core;
mod infra;

use crate::core::CrateVeracityEvaluation;
use crate::core::CrateVeracityLevel;
use crate::infra::cargo::RustProjectDependenciesResolver;
use clap::Parser;
use console::style;
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

    println!();
    println!("Total cargo packages for this project: {}", total_project_packages);
    println!();
    println!("Evaluating veracity for packages. This operation may take some time ...");

    let veracity_checks = cargo_packages
        .into_iter()
        .map(async |package| (package.clone(), veracity_evaluator.evaluate(&package).await))
        .collect::<Vec<_>>();

    let evaluations = futures::future::join_all(veracity_checks).await;

    let total_evaluated_packages = evaluations
        .iter()
        .filter(|(_, check)| check.is_ok())
        .collect::<Vec<_>>()
        .len();

    let total_packages_with_veracity_level = evaluations
        .iter()
        .filter_map(|(_, check)| {
            if let Ok(level) = check {
                Some(level.to_owned().clone())
            } else {
                None
            }
        })
        .filter(|veracity_level| *veracity_level != CrateVeracityLevel::NotAvailable)
        .collect::<Vec<_>>()
        .len();

    println!();
    println!("Packages evaluated : {}", total_evaluated_packages);
    println!(
        "Packages missing veracity checks : {}",
        total_project_packages - total_evaluated_packages
    );
    println!(
        "Packages with existing veracity checks : {}",
        total_packages_with_veracity_level
    );
    println!();

    evaluations
        .iter()
        .for_each(|(package, veracity_check)| match veracity_check {
            Ok(level) => {
                println!("For {} : veracity = {:?} ", package, style(level).cyan());
            },
            Err(_) => {
                println!("For {} : {}", package, style("failed to evaluate").red());
            },
        });

    println!();
    Ok(())
}

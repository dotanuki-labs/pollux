// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::CargoPackage;
use crate::infra::cli::parsing::MainCommands::Evaluate;
use crate::pollux::PolluxTask;
use anyhow::bail;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone)]
enum EvaluationSubject {
    Project,
    Crate,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum CleanupScope {
    Everything,
    Evaluations,
    Packages,
}

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct EvaluateArguments {
    /// Rust project or crate
    #[arg(value_enum)]
    pub subject: EvaluationSubject,

    /// Folder path or crate package url (purl) to evaluate
    pub input: String,
}

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct CheckArguments {
    /// Crate package url (purl) to check
    pub input: String,
}

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct CleanupArguments {
    /// Define the scope of cached data to remove
    #[arg(value_enum)]
    pub mode: CleanupScope,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = false)]
struct CliParser {
    #[command(subcommand)]
    pub command: MainCommands,
}

#[derive(Subcommand)]
enum MainCommands {
    /// Check existing veracity factor for a single package
    Check(CheckArguments),
    /// Clean up existing cached data used by this tool
    Cleanup(CleanupArguments),
    /// Evaluate veracity for a target Rust project or crate
    Evaluate(EvaluateArguments),
}

pub fn parse_arguments() -> anyhow::Result<PolluxTask> {
    let cli = CliParser::parse();

    let task = match cli.command {
        Evaluate(args) => match args.subject {
            EvaluationSubject::Project => {
                let project_path = PathBuf::from(args.input);
                if !project_path.exists() {
                    bail!("pollux.cli : no such file or directory ({:?})", project_path)
                }
                PolluxTask::EvaluateRustProject(project_path)
            },
            EvaluationSubject::Crate => {
                let cargo_package = CargoPackage::try_from(args.input)?;
                PolluxTask::EvaluateRustCrate(cargo_package)
            },
        },
        MainCommands::Cleanup(args) => match args.mode {
            CleanupScope::Everything => PolluxTask::CleanupEverything,
            CleanupScope::Evaluations => PolluxTask::CleanupEvaluations,
            CleanupScope::Packages => PolluxTask::CleanupPackages,
        },
        MainCommands::Check(args) => {
            let cargo_package = CargoPackage::try_from(args.input)?;
            PolluxTask::CheckRustCrate(cargo_package)
        },
    };

    Ok(task)
}

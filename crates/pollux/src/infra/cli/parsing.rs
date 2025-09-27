// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::CargoPackage;
use crate::infra::cli::parsing::MainCommands::Analyse;
use crate::pollux::PolluxTask;
use anyhow::bail;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone)]
enum AnalysisSubject {
    Project,
    Crate,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum CleanupScope {
    Everything,
    AnalysedData,
    PackageSources,
}

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct AnalysisArguments {
    /// Subject specification (Rust project or crate)
    #[arg(value_enum)]
    pub subject: AnalysisSubject,

    /// Folder path or crate package url (purl) to analyse
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
    /// Analyse veracity checks for a target Rust project or crate
    Analyse(AnalysisArguments),
}

pub fn parse_arguments() -> anyhow::Result<PolluxTask> {
    let cli = CliParser::parse();

    let task = match cli.command {
        Analyse(args) => match args.subject {
            AnalysisSubject::Project => {
                let project_path = PathBuf::from(args.input);
                if !project_path.exists() {
                    bail!("pollux.cli : no such file or directory ({:?})", project_path)
                }
                PolluxTask::AnalyseRustProject(project_path)
            },
            AnalysisSubject::Crate => {
                let cargo_package = CargoPackage::try_from(args.input)?;
                PolluxTask::AnalyseRustCrate(cargo_package)
            },
        },
        MainCommands::Cleanup(args) => match args.mode {
            CleanupScope::Everything => PolluxTask::CleanupEverything,
            CleanupScope::AnalysedData => PolluxTask::CleanupAnalysedData,
            CleanupScope::PackageSources => PolluxTask::CleanupPackageSource,
        },
        MainCommands::Check(args) => {
            let cargo_package = CargoPackage::try_from(args.input)?;
            PolluxTask::CheckRustCrate(cargo_package)
        },
    };

    Ok(task)
}

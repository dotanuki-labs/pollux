// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::{CargoPackage, CleanupScope, InquireReportKind};
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

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct AnalysisArguments {
    /// Subject specification (Rust project or crate)
    #[arg(value_enum)]
    pub subject: AnalysisSubject,

    /// Folder path or crate package url (purl) to analyse
    pub input: String,

    /// Whether to use colored output
    #[arg(
        short,
        long,
        action,
        default_value = "false",
        help = "Dont use colors on console output"
    )]
    pub no_color: bool,
}

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct CheckArguments {
    /// Crate package url (purl) to check
    pub input: String,

    /// Whether to use colored output
    #[arg(
        short,
        long,
        action,
        default_value = "false",
        help = "Dont use colors on console output"
    )]
    pub no_color: bool,
}

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct CleanupArguments {
    /// Define the scope of cached data to remove
    #[arg(value_enum)]
    pub mode: CleanupScope,

    /// Whether to use colored output
    #[arg(
        short,
        long,
        action,
        default_value = "false",
        help = "Dont use colors on console output"
    )]
    pub no_color: bool,
}

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct InquiringArguments {
    /// Output type for inquiring reports
    #[arg(short, long, value_enum, default_value = "console")]
    pub output: InquireReportKind,

    /// Whether to use colored output
    #[arg(
        short,
        long,
        action,
        default_value = "false",
        help = "Dont use colors on console output"
    )]
    pub no_color: bool,
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
    /// Evaluate veracity checks for the top packages served by crates.io
    Inquire(InquiringArguments),
}

pub fn parse_arguments() -> anyhow::Result<(PolluxTask, bool)> {
    let cli = CliParser::parse();

    let (task, turnoff_colors) = match cli.command {
        Analyse(args) => match args.subject {
            AnalysisSubject::Project => {
                let project_path = PathBuf::from(args.input);
                if !project_path.exists() {
                    bail!("pollux.cli : no such file or directory ({:?})", project_path)
                }
                (PolluxTask::AnalyseRustProject(project_path), args.no_color)
            },
            AnalysisSubject::Crate => {
                let cargo_package = CargoPackage::try_from(args.input)?;
                (PolluxTask::AnalyseRustCrate(cargo_package), args.no_color)
            },
        },
        MainCommands::Cleanup(args) => match args.mode {
            CleanupScope::Everything => (PolluxTask::CleanupEverything, args.no_color),
            CleanupScope::AnalysedData => (PolluxTask::CleanupAnalysedData, args.no_color),
            CleanupScope::PackageSources => (PolluxTask::CleanupPackageSource, args.no_color),
        },
        MainCommands::Check(args) => {
            let cargo_package = CargoPackage::try_from(args.input)?;
            (PolluxTask::CheckRustCrate(cargo_package), args.no_color)
        },
        MainCommands::Inquire(args) => (PolluxTask::InquirePopularCrates(args.output), args.no_color),
    };

    Ok((task, turnoff_colors))
}

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::CargoPackage;
use crate::infra::cli::MainCommands::Evaluate;
use crate::pollux::PolluxTask;
use anyhow::bail;
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone)]
enum EvaluationSubject {
    Project,
    Crate,
}

#[derive(Args, Debug)]
#[command(version, about, long_about = None)]
struct EvaluateArguments {
    /// Rust project or crate
    #[arg(value_enum)]
    pub subject: EvaluationSubject,

    /// Filesystem path or crate package purl
    pub input: String,
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
    /// Evaluate veracity for a target Rust project
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
    };

    Ok(task)
}

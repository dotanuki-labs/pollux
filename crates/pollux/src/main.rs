// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

mod core;
mod infra;

use crate::core::{CrateInfo, TruthfulnessEvaluator};
use crate::infra::{OssRebuildEvaluator, ReproducibleBuildsEvaluator};
use clap::Parser;
use console::style;
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ProgramArguments {
    #[arg(short, long)]
    name: String,
}

#[tokio::main]
async fn main() {
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

    let trusted_publishing_evaluator = infra::factories::trusted_publishing_evaluator();
    let reproducible_builds_evaluator = ReproducibleBuildsEvaluator::FromOssRebuild(OssRebuildEvaluator {});
    let evaluator = TruthfulnessEvaluator::new(trusted_publishing_evaluator, reproducible_builds_evaluator);

    let parts = arguments.name.split("@").collect::<Vec<_>>();
    let crates_info = CrateInfo::new(parts[0].to_string(), parts[1].to_string());

    let evaluation = evaluator.evaluate(&crates_info).await.unwrap();

    println!("{:?}", style(evaluation).cyan());
}

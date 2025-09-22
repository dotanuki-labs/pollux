// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

mod core;
mod infra;
mod ioc;
mod pollux;

use crate::infra::cli;
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::troubleshooting::setup_troubleshooting();

    let task = cli::parsing::parse_arguments()?;
    let pollux = ioc::create_pollux(task);

    println!("Evaluating veracity for packages. This operation may take some time ...");

    let results = pollux.execute().await?;

    cli::feedback::show_user_feedback(&results);
    Ok(())
}

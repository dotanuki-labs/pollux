// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::PolluxResults;
use console::style;

pub fn show_user_feedback(results: &PolluxResults) {
    let statistics = &results.statistics;

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
}

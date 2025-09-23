// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::PolluxResults;
use console::style;

pub fn show_user_feedback(results: &PolluxResults) {
    let statistics = &results.statistics;
    println!();
    println!("Statistics: ");
    println!("• total packages evaluated : {}", statistics.total_project_packages);
    println!("• missing veracity factors : {}", statistics.without_veracity_level);
    println!("• with existing factors : {}", statistics.with_veracity_level);
    println!();
    println!("Evaluations: ");
    println!();
    results
        .outcomes
        .iter()
        .for_each(|(package, maybe_veracity_check)| match maybe_veracity_check {
            Some(level) => {
                println!("• {} | veracity factors = {} ", package, style(level).cyan());
            },
            None => {
                println!("• {} : {}", package, style("failed to evaluate").red());
            },
        });

    println!();
}

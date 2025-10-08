// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::{
    AnalysisResults, CargoPackage, CleanupScope, CrateVeracityChecks, EcosystemInquiringResults,
};
use comfy_table::Table;
use console::{StyledObject, style};

#[derive(Default)]
pub struct ConsoleReporter {
    use_colors: bool,
}

impl ConsoleReporter {
    pub fn new(use_colors: bool) -> Self {
        Self { use_colors }
    }

    pub fn report_pollux_started(&self) {
        println!();
        println!("Analysing packages. This operation may take some time ...");
    }

    pub fn report_analyser_outcomes(&self, results: &AnalysisResults) {
        let statistics = &results.statistics;
        println!();
        println!("Statistics : ");
        println!();
        println!("• total packages analysed : {}", self.cyan(statistics.total));
        println!(
            "• with provenance attested : {}",
            self.cyan(statistics.provenance_attested)
        );
        println!(
            "• with reproducible builds : {}",
            self.cyan(statistics.reproducible_builds)
        );
        println!();
        println!("Veracity factors : ");
        println!();
        results
            .outcomes
            .iter()
            .for_each(|(package, maybe_veracity_check)| match maybe_veracity_check {
                Some(level) => {
                    println!("• {} ({}) ", package, self.cyan(level));
                },
                None => {
                    println!("• {} : {}", package, self.red("failed to analyse"));
                },
            });

        println!();
    }

    pub fn report_checker_started(&self, cargo_package: &CargoPackage) {
        println!();
        println!("Checking veracity factors for : {}", self.cyan(cargo_package));
        println!();
    }

    pub fn report_checker_outcomes(&self, check: CrateVeracityChecks) {
        println!();

        if let Some(cratesio_link) = check.provenance_evidence {
            println!("• provenance evidence : {}", self.cyan(cratesio_link));
        } else {
            println!("• provenance evidence : {}", self.cyan("not found"));
        }

        if let Some(oss_rebuild_link) = check.reproducibility_evidence {
            println!("• reproducibility evidence : {}", self.cyan(oss_rebuild_link));
        } else {
            println!("• reproducibility evidence : {}", self.cyan("not found"));
        }

        println!();
    }

    pub fn report_cleaning_finished(&self, scope: CleanupScope) {
        let output = match scope {
            CleanupScope::Everything => "All caches removed with success!",
            CleanupScope::AnalysedData => "Cached analysed data removed with success!",
            CleanupScope::PackageSources => "Cached package sources removed with success!",
        };

        println!();
        println!("{}", self.cyan(output));
        println!();
    }

    pub fn report_ecosystem_inquired(&self, results: &EcosystemInquiringResults) {
        println!();
        println!("Statistics : ");
        println!();
        println!("• total packages analysed : {}", self.cyan(results.outcomes.len()));
        println!(
            "• with provenance attested : {} %",
            self.cyan(&results.presence_of_provenance)
        );
        println!(
            "• with reproducible builds : {} %",
            self.cyan(&results.presence_of_reproducibility)
        );
        println!();
        println!("Veracity factors : ");
        println!();

        let mut table = Table::new();
        table.set_header(vec!["Crate name", "Checked versions", "Provenance", "Reproducibility"]);
        results.outcomes.iter().for_each(|outcome| {
            let row = vec![
                outcome.cargo_package.name.as_str(),
                outcome.cargo_package.version.as_str(),
                match outcome.checks.provenance_evidence {
                    None => "no",
                    Some(_) => "yes",
                },
                match outcome.checks.reproducibility_evidence {
                    None => "no",
                    Some(_) => "yes",
                },
            ];

            table.add_row(row);
        });

        println!("{table}");
        println!();
    }

    fn cyan<T>(&self, what: T) -> StyledObject<T> {
        match self.use_colors {
            true => style(what).cyan(),
            false => style(what),
        }
    }

    fn red<T>(&self, what: T) -> StyledObject<T> {
        match self.use_colors {
            true => style(what).cyan(),
            false => style(what),
        }
    }
}

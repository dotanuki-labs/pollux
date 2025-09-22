// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use temp_dir::TempDir;

fn sut() -> Command {
    Command::cargo_bin("pollux").expect("Should be able to create a command")
}

#[test]
fn should_verify_project_from_lockfile() {
    let lockfile_contents = r#"
            version = 3

            [[package]]
            name = "arbitrary"
            version = "1.4.1"
            source = "registry+https://github.com/rust-lang/crates.io-index"
            checksum = "dde20b3d026af13f561bdd0f15edf01fc734f0dafcedbaf42bba506a9517f223"

            [[package]]
            name = "find-msvc-tools"
            version = "0.1.1"
            source = "registry+https://github.com/rust-lang/crates.io-index"
            checksum = "7fd99930f64d146689264c637b5af2f0233a933bef0d8570e2526bf9e083192d"

            [[package]]
            name = "levenshtein"
            version = "1.0.5"
            source = "registry+https://github.com/rust-lang/crates.io-index"
            checksum = "db13adb97ab515a3691f56e4dbab09283d0b86cb45abd991d8634a9d6f501760"
        "#;

    let cargo_project = TempDir::new().expect("Cant create temp dir");

    let lockfile_path = cargo_project.path().join("Cargo.lock");
    fs::write(&lockfile_path, lockfile_contents).expect("failed to cargo manifest file");

    sut()
        .args(["evaluate", "project", cargo_project.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("Packages evaluated : 3"))
        .stdout(contains("For pkg:cargo/arbitrary@1.4.1 : veracity = NotAvailable"))
        .stdout(contains(
            "For pkg:cargo/find-msvc-tools@0.1.1 : veracity = SingleFactor(ProvenanceAttested)",
        ))
        .stdout(contains(
            "For pkg:cargo/levenshtein@1.0.5 : veracity = SingleFactor(ReproducibleBuilds)",
        ));
}

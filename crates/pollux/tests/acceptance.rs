// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use assert_cmd::Command;
use assertor::BooleanAssertion;
use predicates::str::contains;
use std::env::home_dir;
use std::fs;
use temp_dir::TempDir;

fn sut() -> Command {
    Command::cargo_bin("pollux").expect("Should be able to create a command")
}

#[test]
fn should_verify_project_from_path() {
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
        .stdout(contains("total packages evaluated : 3"))
        .stdout(contains("pkg:cargo/arbitrary@1.4.1 | veracity factors = none"))
        .stdout(contains(
            "pkg:cargo/find-msvc-tools@0.1.1 | veracity factors = one(provenance attested)",
        ))
        .stdout(contains(
            "pkg:cargo/levenshtein@1.0.5 | veracity factors = one(reproducible builds)",
        ));
}

#[test]
fn should_verify_project_from_package_purl() {
    sut()
        .args(["evaluate", "crate", "pkg:cargo/serde@1.0.226"])
        .assert()
        .success()
        .stdout(contains("total packages evaluated : 6"))
        .stdout(contains("pkg:cargo/proc-macro2@1.0.101 | veracity factors = none"));
}

#[test]
fn should_cleanup_caches() {
    let lockfile_contents = r#"
            version = 3

            [[package]]
            name = "arbitrary"
            version = "1.4.1"
            source = "registry+https://github.com/rust-lang/crates.io-index"
            checksum = "dde20b3d026af13f561bdd0f15edf01fc734f0dafcedbaf42bba506a9517f223"
        "#;

    let cargo_project = TempDir::new().expect("Cant create temp dir");

    let lockfile_path = cargo_project.path().join("Cargo.lock");
    fs::write(&lockfile_path, lockfile_contents).expect("failed to cargo manifest file");

    sut()
        .args(["evaluate", "project", cargo_project.path().to_str().unwrap()])
        .assert()
        .success();

    sut().args(["cleanup", "everything"]).assert().success();

    let cache_folder = home_dir().unwrap().join(".pollux");

    assertor::assert_that!(cache_folder.exists()).is_false()
}

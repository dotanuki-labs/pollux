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
fn should_analyse_project_from_path() {
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
        .args([
            "analyse",
            "project",
            cargo_project.path().to_str().unwrap(),
            "--no-color",
        ])
        .assert()
        .success()
        .stdout(contains("total packages analysed : 3"))
        .stdout(contains("pkg:cargo/arbitrary@1.4.1 (none)"))
        .stdout(contains("pkg:cargo/find-msvc-tools@0.1.1 (trusted publishing)"))
        .stdout(contains("pkg:cargo/levenshtein@1.0.5 (reproducible builds)"));
}

#[test]
fn should_analyse_project_from_package_purl() {
    sut()
        .args(["analyse", "crate", "pkg:cargo/serde@1.0.226", "--no-color"])
        .assert()
        .success()
        .stdout(contains("total packages analysed : 6"))
        .stdout(contains("pkg:cargo/proc-macro2@1.0.101 (none)"));
}

#[test]
fn should_check_standalone_package_purl() {
    sut()
        .args(["check", "pkg:cargo/bon@3.7.2", "--no-color"])
        .assert()
        .success()
        .stdout(contains(
            "trusted publishing evidence : https://github.com/elastio/bon/actions/runs/17402178810",
        ))
        .stdout(contains("reproducibility evidence : not found"));
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
        .args([
            "analyse",
            "project",
            cargo_project.path().to_str().unwrap(),
            "--no-color",
        ])
        .assert()
        .success();

    sut().args(["cleanup", "everything"]).assert().success();

    let cache_folder = home_dir().unwrap().join(".pollux");

    assertor::assert_that!(cache_folder.exists()).is_false()
}

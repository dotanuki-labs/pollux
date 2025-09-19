// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use assert_cmd::Command;
use std::env::{current_dir, home_dir};

fn sut() -> Command {
    Command::cargo_bin("pollux").expect("Should be able to create a command")
}

fn find_project_root() -> String {
    let current_dir = current_dir().unwrap();
    current_dir // tests
        .parent()
        .unwrap() // crates
        .parent()
        .unwrap() // root
        .to_str()
        .unwrap()
        .to_owned()
}

#[test]
fn should_validate_single_crate_coordinate() {
    let home_dir = home_dir().unwrap();
    let pollux_cache_dir = home_dir.join(".pollux");
    let project_root = find_project_root();
    std::fs::remove_dir_all(&pollux_cache_dir).unwrap_or_else(|_| println!("Nothing to remove"));

    sut()
        .args(["--path", project_root.as_str(), "--name", "bon@3.7.2"])
        .assert()
        .success();

    let cached_file = home_dir
        .join(".pollux")
        .join("cache")
        .join("bon")
        .join("3.7.2")
        .join("veracity-checks.json");

    assert!(cached_file.exists());
}

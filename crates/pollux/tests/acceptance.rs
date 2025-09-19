// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use assert_cmd::Command;
use std::env::home_dir;

fn sut() -> Command {
    Command::cargo_bin("pollux").expect("Should be able to create a command")
}

#[test]
fn should_validate_single_crate_coordinate() {
    let home_dir = home_dir().unwrap();
    let pollux_cache_dir = home_dir.join(".pollux");
    std::fs::remove_dir_all(&pollux_cache_dir).unwrap_or_else(|_| println!("Nothing to remove"));

    sut().args(["--name", "bon@3.7.2"]).assert().success();

    let cached_file = home_dir
        .join(".pollux")
        .join("cache")
        .join("bon")
        .join("3.7.2")
        .join("veracity-checks.json");

    assert!(cached_file.exists());
}

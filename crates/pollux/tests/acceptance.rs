// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use assert_cmd::Command;

fn sut() -> Command {
    Command::cargo_bin("pollux").expect("Should be able to create a command")
}

#[test]
fn should_validate_single_crate_coordinate() {
    let execution = sut().args(["--name", "bon@3.7.2"]).assert();
    execution.success();
}

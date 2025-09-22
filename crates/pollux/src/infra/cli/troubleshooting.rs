// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

pub fn setup_troubleshooting() {
    better_panic::install();
    human_panic::setup_panic!();

    env_logger::builder()
        .format_timestamp(None)
        .format_module_path(false)
        .format_level(false)
        .format_file(false)
        .format_target(false)
        .init();
}

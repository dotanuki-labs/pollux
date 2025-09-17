# Development Guidelines

## Requirements

> [!NOTE]
> We officially support development over `macOS` and `Linux` boxes for now.

This project is written in Rust, and explicitly depends on:

- [rustup](https://rustup.rs/)

Please ensure you have those installed on your system.

## Project setup

This project uses `xtasks` helpers to run common tasks. You can check them by running:

```bash
cargo xtasks

Companion build tasks for pollux

Usage: xtasks <COMMAND>

Commands:
  sources    Inspects sources for formatting and smells
  tests      Runs unit and integration tests
  artifacts  Builds project artifacts (binaries or metadata)
  security   Detects issues with project dependencies
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

We definitely recommend getting started by setting up the latest version of Rust along with
by running:

```bash
rustup show active-toolchain
```

## Code Style

This project adotps a few customizations on top of the standard
[rustfmt](https://rust-lang.github.io/rustfmt)
conventions. In addition, it also provides a
[.editorconfig](https://editorconfig.org/)
file to make it straightforward to get code formatting right on you editor or IDE.

In addition to that, this project uses
[Clippy](https://rust-lang.github.io/rust-clippy)
to catch the most straightforward code smells, not enforcing any additional warnings on
specific patterns, but denying any warnings emitted by `clippy`.

## Commit Conventions

This project does not adopt any specific commit conventions for now.

## Code Conventions

This project encourages
[easy-mode Rust](https://llogiq.github.io/2024/03/28/easy.html)
by default, focusing on simplicity and code readability.

Since this is a very small CLI tool focused on one-off executions, we explicitly avoid advanced
Rust features like `lifetimes` and `macros` for now.

In addition, this project leverages
[anyhow](https://docs.rs/anyhow/latest/anyhow/)
for better error signaling, errors transformations and error propagation.

## Continuous Integration

According to our policies, all code contributions to this project must go through a Pull Request,
and all required status checks must pass.

This project adopts
[GiHub Actions](https://github.com/dotanuki-labs/pollux/actions)
as it CI system. Most of the verifications we'll run on CI are wrapped by the `xtasks` helpers,
as previously mentioned.

In addition to that, we also run a specific `Job` to enforce code quality standards for docs,
Bash scripts and others. In particular, this project enforces the proper open-source license
tracking on all Rust and Bash files.

// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::CargoPackage;
use anyhow::bail;
use cargo_lock::Lockfile;
use std::path::PathBuf;
use std::process::Command;

pub struct RustProjectDependenciesResolver {
    project_root: PathBuf,
}

impl RustProjectDependenciesResolver {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub fn resolve_packages(&self) -> anyhow::Result<Vec<CargoPackage>> {
        let lockfile_path = self.locate_or_generate()?;
        let lockfile = Lockfile::load(lockfile_path)?;
        let crates = lockfile
            .packages
            .into_iter()
            .filter(|pkg| {
                if let Some(source) = &pkg.source {
                    source.is_default_registry()
                } else {
                    false
                }
            })
            .map(|pkg| CargoPackage::new(pkg.name.to_string(), pkg.version.to_string()))
            .collect::<Vec<_>>();

        Ok(crates)
    }

    fn locate_or_generate(&self) -> anyhow::Result<PathBuf> {
        if !self.project_root.join("Cargo.lock").exists() {
            self.generate_lockfile()?
        }

        Ok(self.project_root.join("Cargo.lock"))
    }

    fn generate_lockfile(&self) -> anyhow::Result<()> {
        let cargo_update = Command::new("cargo").arg("update").arg("--workspace").status();

        match cargo_update {
            Ok(status) => {
                if !status.success() {
                    log::error!("cargo update failed: {:?}", status);
                    bail!("error when running `cargo update --workspace`")
                }
            },
            Err(e) => {
                log::error!("cargo update failed: {}", e);
                bail!("couldn't run `cargo update --workspace` to generate a lockfile")
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::CargoPackage;
    use crate::infra::cargo::RustProjectDependenciesResolver;
    use assertor::EqualityAssertion;
    use std::fs;
    use temp_dir::TempDir;

    #[test]
    fn should_detect_multiple_codeowners() {
        let lockfile_contents = r#"
            # Partillay extracted from.
            # https://github.com/xacrimon/dashmap/blob/master/Cargo.lock
            version = 3

            [[package]]
            name = "arbitrary"
            version = "1.4.1"
            source = "registry+https://github.com/rust-lang/crates.io-index"
            checksum = "dde20b3d026af13f561bdd0f15edf01fc734f0dafcedbaf42bba506a9517f223"

            [[package]]
            name = "autocfg"
            version = "1.4.0"
            source = "registry+https://github.com/rust-lang/crates.io-index"
            checksum = "ace50bade8e6234aa140d9a2f552bbee1db4d353f69b8217bc503490fc1a9f26"

            [[package]]
            name = "bitflags"
            version = "2.8.0"
            source = "registry+https://github.com/rust-lang/crates.io-index"
            checksum = "8f68f53c83ab957f72c32642f3868eec03eb974d1fb82e453128456482613d36"

            [[package]]
            name = "cfg-if"
            version = "1.0.0"
            source = "registry+https://github.com/rust-lang/crates.io-index"
            checksum = "baf1de4339761588bc0619e3cbc0120ee582ebb74b53b4efbf79117bd2da40fd"

            [[package]]
            name = "my-project"
            version = "1.0.0"
            dependencies = [
                "arbitrary",
                "autocfg",
                "bitflags",
                "cfg-if"
            ]
        "#;

        let cargo_project = TempDir::new().expect("Cant create temp dir");

        let lockfile_path = cargo_project.path().join("Cargo.lock");
        fs::write(&lockfile_path, lockfile_contents).expect("failed to cargo manifest file");

        let resolver = RustProjectDependenciesResolver {
            project_root: cargo_project.path().to_path_buf(),
        };

        let dependencies = resolver.resolve_packages().expect("resolve_dependencies failed");

        let expected_packages = vec![
            CargoPackage::with("arbitrary", "1.4.1"),
            CargoPackage::with("autocfg", "1.4.0"),
            CargoPackage::with("bitflags", "2.8.0"),
            CargoPackage::with("cfg-if", "1.0.0"),
        ];

        assertor::assert_that!(dependencies).is_equal_to(expected_packages)
    }
}

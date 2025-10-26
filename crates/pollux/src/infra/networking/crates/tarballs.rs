// Copyright 2025 Dotanuki Labs
// SPDX-License-Identifier: MIT

use crate::core::models::CargoPackage;
use crate::infra::caching::CacheManager;
use crate::infra::networking::crates::registry::CratesDotIOClient;
use anyhow::Context;
use camino::Utf8PathBuf;
use decompress::{Decompressor, ExtractOptsBuilder, decompressors};
use std::fs;

pub struct CrateArchiveDownloader {
    cratesio_client: CratesDotIOClient,
    cache_manager: CacheManager,
}

impl CrateArchiveDownloader {
    pub fn new(cratesio_client: CratesDotIOClient, cache_manager: CacheManager) -> Self {
        Self {
            cratesio_client,
            cache_manager,
        }
    }

    pub async fn download_extract(&self, target_package: &CargoPackage) -> anyhow::Result<Utf8PathBuf> {
        log::info!("[pollux.cargo] downloading package : {}", target_package.name);

        let downloaded = self
            .cratesio_client
            .get_crate_tarball(&target_package.name, &target_package.version)
            .await?;

        let project_dir = self.cache_manager.temporary_downloads_dir().join(&target_package.name);

        match fs::remove_dir_all(&project_dir) {
            Ok(_) => log::info!(
                "[pollux.cargo] removed previous downloaded archive for {}",
                &target_package
            ),
            Err(_) => log::info!(
                "[pollux.cargo] cannot remove previous downloaded archive for : {}",
                &target_package
            ),
        };

        fs::create_dir_all(&project_dir).context("failed to crate download folder")?;
        let tarball_path = project_dir.join("crate.tar.gz");
        fs::write(&tarball_path, downloaded).context("failed to save crate archive")?;

        log::info!("[pollux.cargo] decompressing package : {}", &target_package);

        // we levaregate the targz format as per what similar crates like
        // https://crates.io/crates/crate_untar also do
        let decompressor = decompressors::targz::Targz::default();
        let extraction_opts = ExtractOptsBuilder::default().build()?;
        decompressor.decompress(&tarball_path, &project_dir, &extraction_opts)?;

        // by convention, a tarball for a package pkg:cargo/crate@x.y.z
        // will extract to a crate-x.y.z folder
        let extraction_path = format!("{}-{}", target_package.name, target_package.version);
        let output_dir = project_dir.join(extraction_path);

        // we remove the downloaded tarball after
        fs::remove_file(tarball_path).context("failed to remove tarball")?;

        log::info!("[pollux.cargo] downloaded and extracted files for {}", &target_package);
        let output_dir = Utf8PathBuf::try_from(output_dir).context("cannot get an utf-8 path")?;
        Ok(output_dir)
    }
}

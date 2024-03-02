/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use anyhow::{anyhow, Context, Result};
use clap::ArgMatches;
use espanso_package::{Archiver, PackageSpecifier, ProviderOptions, SaveOptions, StoredPackage};
use espanso_path::Paths;

use crate::{error_eprintln, info_println, warn_eprintln};

pub enum UpdateResults {
    Success,
    PartialFailure,
}

pub fn update_package(paths: &Paths, matches: &ArgMatches) -> Result<UpdateResults> {
    let package_name = matches
        .value_of("package_name")
        .ok_or_else(|| anyhow!("missing package name"))?;

    let archiver =
        espanso_package::get_archiver(&paths.packages).context("unable to get package archiver")?;

    let packages_to_update = if package_name == "all" {
        let packages = archiver.list()?;
        info_println!("updating {} packages", packages.len());

        packages
            .into_iter()
            .map(|package| match package {
                StoredPackage::Legacy(legacy) => legacy.name,
                StoredPackage::Modern(modern) => modern.manifest.name,
            })
            .collect()
    } else {
        vec![package_name.to_owned()]
    };

    let mut update_errors = Vec::new();

    for package_name in &packages_to_update {
        if let Err(err) = perform_package_update(paths, &*archiver, package_name) {
            error_eprintln!("error updating package '{}': {:?}", package_name, err);
            update_errors.push(err);
        }
    }

    if update_errors.is_empty() {
        Ok(UpdateResults::Success)
    } else if packages_to_update.len() == update_errors.len() {
        Err(update_errors.pop().expect("unable to extract error"))
    } else {
        Ok(UpdateResults::PartialFailure)
    }
}

fn perform_package_update(
    paths: &Paths,
    archiver: &dyn Archiver,
    package_name: &str,
) -> Result<()> {
    info_println!("updating package: {}", package_name);

    let package = archiver.get(package_name)?;

    let (package_specifier, old_version) = match package {
        StoredPackage::Legacy(legacy) => {
            warn_eprintln!(
        "detected legacy package '{}' without source information, pulling from espanso hub.",
        legacy.name
      );
            (
                PackageSpecifier {
                    name: legacy.name,
                    ..Default::default()
                },
                None,
            )
        }
        StoredPackage::Modern(modern) => ((&modern).into(), Some(modern.manifest.version)),
    };

    let package_provider = espanso_package::get_provider(
        &package_specifier,
        &paths.runtime,
        &ProviderOptions::default(),
    )
    .context("unable to obtain compatible package provider")?;

    info_println!("using package provider: {}", package_provider.name());

    let new_package = package_provider.download(&package_specifier)?;

    if new_package.version() == old_version.unwrap_or_default() {
        info_println!("already up to date!");
        return Ok(());
    }

    archiver
        .save(
            &*new_package,
            &package_specifier,
            &SaveOptions {
                overwrite_existing: true,
            },
        )
        .context("unable to save package")?;

    info_println!(
        "updated package '{}' to version: {}",
        new_package.name(),
        new_package.version()
    );

    Ok(())
}

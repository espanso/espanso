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

use anyhow::{Context, Result};
use clap::ArgMatches;
use espanso_package::StoredPackage;
use espanso_path::Paths;

use crate::info_println;

pub fn list_packages(paths: &Paths, _: &ArgMatches) -> Result<()> {
  let archiver =
    espanso_package::get_archiver(&paths.packages).context("unable to get package archiver")?;

  let packages = archiver.list().context("unable to list packages")?;

  if packages.is_empty() {
    info_println!("No packages found!");
    return Ok(());
  }

  info_println!("Installed packages:");
  info_println!("");

  for package in packages {
    match package {
      StoredPackage::Legacy(legacy) => {
        info_println!("- {} (legacy)", legacy.name);
      },
      StoredPackage::Modern(package) => {
        info_println!("- {} - version: {} ({})", package.manifest.name, package.manifest.version, package.source);
      },
    }
  }

  Ok(())
}

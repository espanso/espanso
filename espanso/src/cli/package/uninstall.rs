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
use espanso_path::Paths;

use crate::info_println;

pub fn uninstall_package(paths: &Paths, matches: &ArgMatches) -> Result<()> {
  let package_name = matches
    .value_of("package_name")
    .ok_or_else(|| anyhow!("missing package name"))?;

  let archiver =
    espanso_package::get_archiver(&paths.packages).context("unable to get package archiver")?;

  archiver
    .delete(package_name)
    .context("unable to delete package")?;

  info_println!("package '{}' uninstalled!", package_name);

  Ok(())
}

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
use espanso_package::{PackageSpecifier, SaveOptions};
use espanso_path::Paths;

use crate::info_println;

pub fn install_package(paths: &Paths, matches: &ArgMatches) -> Result<()> {
  let package_name = matches
    .value_of("package_name")
    .ok_or_else(|| anyhow!("missing package name"))?;
  let version = matches.value_of("version");
  let force = matches.is_present("force");

  info_println!(
    "installing package: {} - version: {}",
    package_name,
    version.unwrap_or("latest")
  );

  let package_specifier = if let Some(git_repo) = matches.value_of("git") {
    let git_branch = matches.value_of("git-branch");
    let use_native_git = matches.is_present("use-native-git");

    PackageSpecifier {
      name: package_name.to_string(),
      version: version.map(String::from),
      git_repo_url: Some(git_repo.to_string()),
      git_branch: git_branch.map(String::from),
      use_native_git,
    }
  } else {
    // Install from the hub

    PackageSpecifier {
      name: package_name.to_string(),
      version: version.map(String::from),
      ..Default::default()
    }
  };

  // TODO: if git is specified, make sure external is as well (or warn otherwise)

  let package_provider = espanso_package::get_provider(&package_specifier)
    .context("unable to obtain compatible package provider")?;

  info_println!("using package provider: {}", package_provider.name());

  let package = package_provider.download(&package_specifier)?;

  info_println!(
    "found package: {} - version: {}",
    package.name(),
    package.version()
  );

  let archiver =
    espanso_package::get_archiver(&paths.packages).context("unable to get package archiver")?;

  archiver.save(&*package, &package_specifier, &SaveOptions {
    overwrite_existing: force,
  }).context("unable to save package")?;

  info_println!("package installed!");

  Ok(())
}

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

use super::PackageProvider;
use crate::{
  package::DefaultPackage, resolver::resolve_package, util::download::read_string_from_url,
  Package, PackageSpecifier,
};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

pub const ESPANSO_HUB_PACKAGE_INDEX_URL: &str =
  "https://github.com/espanso/hub/releases/latest/download/package_index.json";

pub struct EspansoHubPackageProvider {}

impl EspansoHubPackageProvider {
  pub fn new() -> Self {
    Self {}
  }
}

impl PackageProvider for EspansoHubPackageProvider {
  fn name(&self) -> String {
    "espanso-hub".to_string()
  }

  fn download(&self, package: &PackageSpecifier) -> Result<Box<dyn Package>> {
    // TODO: pass index update flag
    let index = self
      .get_index(true)
      .context("unable to get package index from espanso hub")?;

    let package_info = index
      .get_package(&package.name, package.version.as_deref())
      .ok_or_else(|| {
        anyhow!(
          "unable to find package '{}@{}' in the espanso hub",
          package.name,
          package.version.as_deref().unwrap_or("latest")
        )
      })?;

    let archive_sha256 = read_string_from_url(&package_info.archive_sha256_url)
      .context("unable to read archive sha256 signature")?;

    let temp_dir = tempdir::TempDir::new("espanso-package-download")?;

    crate::util::download::download_and_extract_zip_verify_sha256(
      &package_info.archive_url,
      temp_dir.path(),
      Some(&archive_sha256),
    )?;

    let resolved_package =
      resolve_package(temp_dir.path(), &package.name, package.version.as_deref())?;

    let package = DefaultPackage::new(
      resolved_package.manifest,
      temp_dir,
      resolved_package.base_dir,
    );

    Ok(Box::new(package))
  }
}

impl EspansoHubPackageProvider {
  fn get_index(&self, _force_update: bool) -> Result<PackageIndex> {
    // TODO: if force_update is false, we should try to use a locally-cached version of the package index
    self.download_index()
  }

  fn download_index(&self) -> Result<PackageIndex> {
    info_println!("fetching package index...");
    let json_body = read_string_from_url(ESPANSO_HUB_PACKAGE_INDEX_URL)?;

    let index: PackageIndex = serde_json::from_str(&json_body)?;

    Ok(index)
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct PackageIndex {
  last_update: u64,
  packages: Vec<PackageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageInfo {
  name: String,
  title: String,
  author: String,
  description: String,
  version: String,

  archive_url: String,
  archive_sha256_url: String,
}

impl PackageIndex {
  fn get_package(&self, name: &str, version: Option<&str>) -> Option<PackageInfo> {
    let mut matching_packages: Vec<PackageInfo> = self
      .packages
      .iter()
      .filter(|package| package.name == name)
      .cloned()
      .collect();

    matching_packages.sort_by(|a, b| natord::compare(&a.version, &b.version));

    if let Some(explicit_version) = version {
      matching_packages
        .into_iter()
        .find(|package| package.version == explicit_version)
    } else {
      matching_packages.into_iter().last()
    }
  }
}
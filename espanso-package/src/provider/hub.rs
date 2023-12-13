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

use std::{
  path::{Path, PathBuf},
  time::UNIX_EPOCH,
};

use super::PackageProvider;
use crate::{
  package::DefaultPackage, resolver::resolve_package, util::download::read_string_from_url,
  Package, PackageSpecifier,
};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

pub const ESPANSO_HUB_PACKAGE_INDEX_URL: &str =
  "https://github.com/espanso/hub/releases/latest/download/package_index.json";

const PACKAGE_INDEX_CACHE_FILE: &str = "package_index_cache.json";
const PACKAGE_INDEX_CACHE_INVALIDATION_SECONDS: u64 = 60 * 60;

pub struct EspansoHubPackageProvider {
  runtime_dir: PathBuf,
  force_index_update: bool,
}

impl EspansoHubPackageProvider {
  pub fn new(runtime_dir: &Path, force_index_update: bool) -> Self {
    Self {
      runtime_dir: runtime_dir.to_path_buf(),
      force_index_update,
    }
  }
}

impl PackageProvider for EspansoHubPackageProvider {
  fn name(&self) -> String {
    "espanso-hub".to_string()
  }

  fn download(&self, package: &PackageSpecifier) -> Result<Box<dyn Package>> {
    let index = self
      .get_index(self.force_index_update)
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
  fn get_index(&self, force_update: bool) -> Result<PackageIndex> {
    let old_index = self.get_index_from_cache()?;

    if let Some(old_index) = old_index {
      if !force_update {
        let current_time = std::time::SystemTime::now().duration_since(UNIX_EPOCH)?;
        let current_unix = current_time.as_secs();
        if old_index.cached_at >= (current_unix - PACKAGE_INDEX_CACHE_INVALIDATION_SECONDS) {
          info_println!("using cached package index");
          return Ok(old_index.index);
        }
      }
    }

    let new_index = EspansoHubPackageProvider::download_index()?;
    self.save_index_to_cache(new_index.clone())?;
    Ok(new_index)
  }

  fn download_index() -> Result<PackageIndex> {
    info_println!("fetching package index...");
    let json_body = read_string_from_url(ESPANSO_HUB_PACKAGE_INDEX_URL)?;

    let index: PackageIndex = serde_json::from_str(&json_body)?;

    Ok(index)
  }

  fn get_index_from_cache(&self) -> Result<Option<CachedPackageIndex>> {
    let target_file = self.runtime_dir.join(PACKAGE_INDEX_CACHE_FILE);
    if !target_file.is_file() {
      return Ok(None);
    }

    let content =
      std::fs::read_to_string(&target_file).context("unable to read package index cache")?;
    let index: CachedPackageIndex = serde_json::from_str(&content)?;
    Ok(Some(index))
  }

  fn save_index_to_cache(&self, index: PackageIndex) -> Result<()> {
    let target_file = self.runtime_dir.join(PACKAGE_INDEX_CACHE_FILE);
    let current_time = std::time::SystemTime::now().duration_since(UNIX_EPOCH)?;
    let current_unix = current_time.as_secs();
    let cached_index = CachedPackageIndex {
      cached_at: current_unix,
      index,
    };
    let serialized = serde_json::to_string(&cached_index)?;
    std::fs::write(target_file, serialized)?;
    Ok(())
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedPackageIndex {
  cached_at: u64,
  index: PackageIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

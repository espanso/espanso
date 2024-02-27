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

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{manifest::Manifest, Package, PackageSpecifier};

pub mod default;
mod read;
mod util;

pub const PACKAGE_SOURCE_FILE: &str = "_pkgsource.yml";

#[derive(Debug, PartialEq, Eq)]
pub struct ArchivedPackage {
    // Metadata
    pub manifest: Manifest,

    // Package source information (needed to update)
    pub source: PackageSource,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LegacyPackage {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StoredPackage {
    Legacy(LegacyPackage),
    Modern(ArchivedPackage),
}

pub trait Archiver {
    fn get(&self, name: &str) -> Result<StoredPackage>;
    fn save(
        &self,
        package: &dyn Package,
        specifier: &PackageSpecifier,
        save_options: &SaveOptions,
    ) -> Result<ArchivedPackage>;
    fn list(&self) -> Result<Vec<StoredPackage>>;
    fn delete(&self, name: &str) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct SaveOptions {
    pub overwrite_existing: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageSource {
    Hub,
    Git {
        repo_url: String,
        repo_branch: Option<String>,
        use_native_git: bool,
    },
}

impl PackageSource {
    pub fn parse(source_path: &Path) -> Result<Self> {
        let source_str = std::fs::read_to_string(source_path)?;
        Ok(serde_yaml::from_str(&source_str)?)
    }
}

impl std::fmt::Display for PackageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageSource::Hub => write!(f, "espanso-hub"),
            PackageSource::Git {
                repo_url,
                repo_branch: _,
                use_native_git: _,
            } => write!(f, "git: {repo_url}"),
        }
    }
}

impl From<&PackageSpecifier> for PackageSource {
    fn from(package: &PackageSpecifier) -> Self {
        if let Some(git_repo_url) = package.git_repo_url.as_deref() {
            Self::Git {
                repo_url: git_repo_url.to_string(),
                repo_branch: package.git_branch.clone(),
                use_native_git: package.use_native_git,
            }
        } else {
            Self::Hub
        }
    }
}

impl From<&ArchivedPackage> for PackageSpecifier {
    fn from(package: &ArchivedPackage) -> Self {
        match &package.source {
            PackageSource::Hub => PackageSpecifier {
                name: package.manifest.name.to_string(),
                ..Default::default()
            },
            PackageSource::Git {
                repo_url,
                repo_branch,
                use_native_git,
            } => PackageSpecifier {
                name: package.manifest.name.to_string(),
                git_repo_url: Some(repo_url.to_string()),
                git_branch: repo_branch.clone(),
                use_native_git: *use_native_git,
                ..Default::default()
            },
        }
    }
}

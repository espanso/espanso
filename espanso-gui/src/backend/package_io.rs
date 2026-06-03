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

//! Package management operations for the GUI.
//!
//! Wraps espanso-package for list/install/uninstall operations,
//! and fetches the Hub index directly for browsing/search.

use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// === Hub package index types ===

const HUB_INDEX_URL: &str =
    "https://github.com/espanso/hub/releases/latest/download/package_index.json";
const INDEX_CACHE_FILE: &str = "package_index_cache.json";
const INDEX_CACHE_TTL_SECS: u64 = 3600; // 1 hour

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubPackageInfo {
    pub name: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub version: String,
    pub archive_url: String,
    pub archive_sha256_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageIndex {
    last_update: u64,
    packages: Vec<HubPackageInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedIndex {
    cached_at: u64,
    index: PackageIndex,
}

/// Load the package index, using a local cache.
pub fn load_hub_index(runtime_dir: &Path) -> Result<Vec<HubPackageInfo>> {
    // Check cache first
    let cache_file = runtime_dir.join(INDEX_CACHE_FILE);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if cache_file.exists() {
        if let Ok(contents) = std::fs::read_to_string(&cache_file) {
            if let Ok(cached) = serde_json::from_str::<CachedIndex>(&contents) {
                if now < cached.cached_at + INDEX_CACHE_TTL_SECS {
                    return Ok(cached.index.packages);
                }
            }
        }
    }

    // Fetch fresh index
    info!("Fetching Hub package index...");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")?;

    let json = client
        .get(HUB_INDEX_URL)
        .send()
        .context("Failed to fetch Hub index")?
        .text()
        .context("Failed to read Hub index response")?;

    let index: PackageIndex =
        serde_json::from_str(&json).context("Failed to parse Hub index JSON")?;

    // Update cache
    let cached = CachedIndex {
        cached_at: now,
        index: index.clone(),
    };
    if let Err(e) = std::fs::create_dir_all(runtime_dir) {
        log::warn!("Cannot create runtime dir for cache: {}", e);
    }
    match serde_json::to_string(&cached) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&cache_file, &json) {
                log::warn!("Failed to write Hub index cache: {}", e);
            }
        }
        Err(e) => log::warn!("Failed to serialize Hub index cache: {}", e),
    }

    Ok(index.packages)
}

// === Installed packages ===

#[derive(Debug, Clone)]
pub struct InstalledPackageInfo {
    pub name: String,
    pub title: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub source: String, // "Hub" or "Git: <url>"
}

/// List all installed packages.
pub fn list_installed(packages_dir: &Path) -> Result<Vec<InstalledPackageInfo>> {
    let archiver = espanso_package::get_archiver(packages_dir)
        .context("Failed to create package archiver")?;

    let stored = archiver.list().context("Failed to list installed packages")?;

    let mut results = Vec::new();
    for pkg in stored {
        match pkg {
            espanso_package::StoredPackage::Modern(archived) => {
                results.push(InstalledPackageInfo {
                    name: archived.manifest.name,
                    title: archived.manifest.title,
                    description: archived.manifest.description,
                    version: archived.manifest.version,
                    author: archived.manifest.author,
                    source: archived.source.to_string(),
                });
            }
            espanso_package::StoredPackage::Legacy(legacy) => {
                results.push(InstalledPackageInfo {
                    name: legacy.name.clone(),
                    title: legacy.name.clone(),
                    description: String::new(),
                    version: "unknown".to_string(),
                    author: "unknown".to_string(),
                    source: "legacy".to_string(),
                });
            }
        }
    }

    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

/// Install a package from the Hub.
pub fn install_from_hub(
    packages_dir: &Path,
    runtime_dir: &Path,
    name: &str,
) -> Result<String> {
    let spec = espanso_package::PackageSpecifier {
        name: name.to_string(),
        ..Default::default()
    };

    let provider = espanso_package::get_provider(
        &spec,
        runtime_dir,
        &espanso_package::ProviderOptions::default(),
    )?;

    info!("Downloading package '{}'...", name);
    let package = provider
        .download(&spec)
        .with_context(|| format!("Failed to download package '{}'", name))?;

    let archiver = espanso_package::get_archiver(packages_dir)
        .context("Failed to create archiver")?;

    archiver.save(
        package.as_ref(),
        &spec,
        &espanso_package::SaveOptions {
            overwrite_existing: true,
        },
    )?;

    Ok(format!("Package '{}' installed", name))
}

/// Install a package from a Git repository.
pub fn install_from_git(
    packages_dir: &Path,
    runtime_dir: &Path,
    repo_url: &str,
    branch: Option<&str>,
) -> Result<String> {
    let spec = espanso_package::PackageSpecifier {
        name: String::new(), // Will be filled from manifest
        git_repo_url: Some(repo_url.to_string()),
        git_branch: branch.map(|s| s.to_string()),
        use_native_git: false,
        ..Default::default()
    };

    let provider = espanso_package::get_provider(
        &spec,
        runtime_dir,
        &espanso_package::ProviderOptions::default(),
    )?;

    info!("Downloading package from git: {}...", repo_url);
    let package = provider
        .download(&spec)
        .with_context(|| format!("Failed to download from git: {}", repo_url))?;

    let archiver = espanso_package::get_archiver(packages_dir)
        .context("Failed to create archiver")?;

    archiver.save(
        package.as_ref(),
        &spec,
        &espanso_package::SaveOptions {
            overwrite_existing: true,
        },
    )?;

    Ok(format!("Package from '{}' installed", repo_url))
}

/// Uninstall a package.
pub fn uninstall_package(packages_dir: &Path, name: &str) -> Result<String> {
    let archiver = espanso_package::get_archiver(packages_dir)
        .context("Failed to create archiver")?;

    archiver
        .delete(name)
        .with_context(|| format!("Failed to uninstall '{}'", name))?;

    Ok(format!("Package '{}' uninstalled", name))
}

/// Check which installed packages have updates available on the Hub.
pub fn check_updates(
    packages_dir: &Path,
    runtime_dir: &Path,
) -> Result<Vec<(InstalledPackageInfo, String)>> {
    let installed = list_installed(packages_dir)?;
    let hub_index = match load_hub_index(runtime_dir) {
        Ok(idx) => idx,
        Err(e) => {
            log::warn!("Cannot check updates: {}", e);
            return Ok(Vec::new());
        }
    };

    let mut updates = Vec::new();
    for pkg in installed {
        if let Some(hub_pkg) = hub_index.iter().find(|h| h.name == pkg.name) {
            if hub_pkg.version != pkg.version {
                // Simple string comparison; natord would be better
                updates.push((pkg, hub_pkg.version.clone()));
            }
        }
    }

    Ok(updates)
}

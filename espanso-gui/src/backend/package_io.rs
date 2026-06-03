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
//! This module wraps espanso-package to provide package browsing,
//! installation, and management.

use anyhow::Result;
use log::info;

/// Information about an installed package.
#[derive(Debug, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub author: String,
}

/// Information about a package available on the Hub.
#[derive(Debug, Clone)]
pub struct HubPackage {
    pub name: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub stars: u32,
    pub is_installed: bool,
    pub installed_version: Option<String>,
    pub has_update: bool,
}

/// Result of a package operation.
#[derive(Debug)]
pub enum PackageOpResult {
    Success(String),
    Error(String),
}

/// List installed packages.
pub fn list_installed(_packages_dir: &std::path::Path) -> Result<Vec<InstalledPackage>> {
    // TODO: Use espanso_package::Archiver to list installed packages
    info!("Listing installed packages (stub)");
    // Stub: return empty list for now
    Ok(Vec::new())
}

/// Search the Hub for packages.
pub fn search_hub(_query: &str) -> Result<Vec<HubPackage>> {
    // TODO: Use espanso_package::Provider to fetch Hub index
    info!("Searching Hub for '{}' (stub)", _query);
    // Stub: return empty list for now
    Ok(Vec::new())
}

/// Install a package from the Hub.
pub fn install_package(_packages_dir: &std::path::Path, _name: &str) -> Result<PackageOpResult> {
    // TODO: Use espanso_package::Provider::download + Archiver::save
    info!("Installing package '{}' (stub)", _name);
    // Stub
    Ok(PackageOpResult::Success(format!(
        "Package '{}' installed successfully",
        _name
    )))
}

/// Uninstall a package.
pub fn uninstall_package(_packages_dir: &std::path::Path, _name: &str) -> Result<PackageOpResult> {
    // TODO: Use espanso_package::Archiver::delete
    info!("Uninstalling package '{}' (stub)", _name);
    // Stub
    Ok(PackageOpResult::Success(format!(
        "Package '{}' uninstalled successfully",
        _name
    )))
}

/// Check for package updates.
pub fn check_updates(
    _packages_dir: &std::path::Path,
) -> Result<Vec<(InstalledPackage, String)>> {
    // TODO: Compare installed versions against Hub index
    info!("Checking for package updates (stub)");
    // Stub
    Ok(Vec::new())
}

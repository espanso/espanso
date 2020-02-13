/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

pub(crate) mod default;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub trait PackageManager {
    fn is_index_outdated(&self) -> bool;
    fn update_index(&mut self, force: bool) -> Result<UpdateResult, Box<dyn Error>>;

    fn get_package(&self, name: &str) -> Option<Package>;

    fn install_package(&self, name: &str) -> Result<InstallResult, Box<dyn Error>>;
    fn install_package_from_repo(
        &self,
        name: &str,
        repo_url: &str,
    ) -> Result<InstallResult, Box<dyn Error>>;

    fn remove_package(&self, name: &str) -> Result<RemoveResult, Box<dyn Error>>;

    fn list_local_packages(&self) -> Vec<Package>;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Package {
    pub name: String,
    pub title: String,
    pub version: String,
    pub repo: String,
    pub desc: String,
    pub author: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PackageIndex {
    #[serde(rename = "lastUpdate")]
    pub last_update: u64,

    pub packages: Vec<Package>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UpdateResult {
    NotOutdated,
    Updated,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InstallResult {
    NotFoundInIndex,
    NotFoundInRepo,
    UnableToParsePackageInfo,
    MissingPackageVersion,
    AlreadyInstalled,
    Installed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RemoveResult {
    NotFound,
    Removed,
}

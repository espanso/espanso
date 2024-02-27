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
use crate::{package::DefaultPackage, resolver::resolve_package, Package, PackageSpecifier};
use anyhow::Result;

pub struct GitLabPackageProvider {
    repo_author: String,
    repo_name: String,
    repo_branch: String,
}

impl GitLabPackageProvider {
    pub fn new(repo_author: String, repo_name: String, repo_branch: String) -> Self {
        Self {
            repo_author,
            repo_name,
            repo_branch,
        }
    }
}

impl PackageProvider for GitLabPackageProvider {
    fn name(&self) -> String {
        "gitlab".to_string()
    }

    fn download(&self, package: &PackageSpecifier) -> Result<Box<dyn Package>> {
        let download_url = format!(
            "https://gitlab.com/{}/{}/-/archive/{}/{}-{}.zip",
            &self.repo_author,
            &self.repo_name,
            &self.repo_branch,
            &self.repo_name,
            &self.repo_branch
        );

        let temp_dir = tempdir::TempDir::new("espanso-package-download")?;

        crate::util::download::download_and_extract_zip(&download_url, temp_dir.path())?;

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

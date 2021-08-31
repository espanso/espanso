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

use crate::{
  package::DefaultPackage, resolver::resolve_package, Package, PackageProvider, PackageSpecifier,
};
use anyhow::{Result};

pub struct GitHubPackageProvider {
  repo_author: String,
  repo_name: String,
  repo_branch: String,
}

impl GitHubPackageProvider {
  pub fn new(repo_author: String, repo_name: String, repo_branch: String) -> Self {
    Self {
      repo_author,
      repo_name,
      repo_branch,
    }
  }
}

impl PackageProvider for GitHubPackageProvider {
  fn download(&self, package: &PackageSpecifier) -> Result<Box<dyn Package>> {
    let download_url = format!(
      "https://github.com/{}/{}/archive/refs/heads/{}.zip",
      &self.repo_author, &self.repo_name, &self.repo_branch
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[ignore]
  fn test_download_github_package() {
    let provider = GitHubPackageProvider::new(
      "espanso".to_string(),
      "dummy-package".to_string(),
      "main".to_string(),
    );
    provider
      .download(&PackageSpecifier {
        name: "dummy-package".to_string(),
        version: None,
        git_repo_url: Some("https://github.com/espanso/dummy-package".to_string()),
        git_branch: None,
      })
      .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(300));
  }
}

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
use anyhow::{anyhow, bail, Context, Result};
use std::{path::Path, process::Command};

pub struct GitPackageProvider {}

impl GitPackageProvider {
  pub fn new() -> Self {
    Self {}
  }

  fn is_git_installed() -> bool {
    if let Ok(output) = Command::new("git").arg("--version").output() {
      if output.status.success() {
        return true;
      }
    }

    false
  }

  fn clone_repo(dest_dir: &Path, repo_url: &str, repo_branch: Option<&str>) -> Result<()> {
    let mut args = vec!["clone"];

    if let Some(branch) = repo_branch {
      args.push("-b");
      args.push(branch);
    }

    args.push(repo_url);

    let dest_dir_str = dest_dir.to_string_lossy().to_string();
    args.push(&dest_dir_str);

    let output = Command::new("git")
      .args(&args)
      .output()
      .context("git command reported error")?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      bail!("git command exited with non-zero status: {}", stderr);
    }
    Ok(())
  }
}

impl PackageProvider for GitPackageProvider {
  fn name(&self) -> String {
    "git".to_string()
  }

  fn download(&self, package: &PackageSpecifier) -> Result<Box<dyn Package>> {
    if !Self::is_git_installed() {
      bail!("unable to invoke 'git' command, please make sure it is installed and visible in PATH");
    }

    let repo_url = package
      .git_repo_url
      .as_deref()
      .ok_or_else(|| anyhow!("missing git repository url"))?;
    let repo_branch = package.git_branch.as_deref();

    let temp_dir = tempdir::TempDir::new("espanso-package-download")?;

    Self::clone_repo(temp_dir.path(), repo_url, repo_branch)?;

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

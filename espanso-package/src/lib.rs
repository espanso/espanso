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

use anyhow::{Result, bail};
use thiserror::Error;

mod manifest;
mod package;
mod provider;
mod resolver;
mod util;

#[derive(Debug, Default)]
pub struct PackageSpecifier {
  pub name: String,
  pub version: Option<String>,

  // Source information
  pub git_repo_url: Option<String>,
  pub git_branch: Option<String>,
}

pub trait Package {
  // Manifest
  fn name(&self) -> &str;
  fn title(&self) -> &str;
  fn description(&self) -> &str;
  fn version(&self) -> &str;
  fn author(&self) -> &str;

  // Directory containing the package files
  fn location(&self) -> &Path;
}

pub trait PackageProvider {
  fn download(&self, package: &PackageSpecifier) -> Result<Box<dyn Package>>;
  // TODO: fn check update available? (probably should be only available in the hub)
}

// TODO: once the download is completed, avoid copying files beginning with "."

#[derive(Error, Debug)]
pub enum PackageResolutionError {
  #[error("package not found")]
  PackageNotFound,
}

pub fn get_provider(package: &PackageSpecifier) -> Result<Box<dyn PackageProvider>> {
  if let Some(git_repo_url) = package.git_repo_url.as_deref() {
    let matches_known_hosts = if let Some(github_parts) = util::github::extract_github_url_parts(git_repo_url) {
      if let Some(repo_scheme) =
        util::github::resolve_repo_scheme(github_parts, package.git_branch.as_deref())?
      {
        return Ok(Box::new(provider::github::GitHubPackageProvider::new(
          repo_scheme.author,
          repo_scheme.name,
          repo_scheme.branch,
        )));
      }

      true
    } else if let Some(gitlab_parts) = util::gitlab::extract_gitlab_url_parts(git_repo_url) {
      panic!("GitLab is not supported yet!");
      todo!();

      true
    } else {
      false
    };

    // Git repository seems to be in one of the known hosts, but the direct methods
    // couldn't retrieve its content. This might happen with private repos (as they are not
    // available to non-authenticated requests), so we check if a "git ls-remote" command 
    // is able to access it.
    if matches_known_hosts && !util::git::is_private_repo(git_repo_url) {
      bail!("could not access repository: {}, make sure it exists and that you have the necessary access rights.");
    }

    // Git repository is neither on Github or Gitlab
    // OR it's a private repository, which means we can't use the direct method
    // (because it's not authenticated)
    Ok(Box::new(provider::git::GitPackageProvider::new()))
  } else {
    // TODO: use espanso-hub method
    todo!();
  }
}

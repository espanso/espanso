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
use thiserror::Error;

mod resolver;

#[derive(Debug, Default)]
pub struct PackageSpecifier {
  pub name: String,
  pub version: Option<String>,
  
  // Source information
  pub git_repo_url: Option<String>,
  pub git_branch: Option<String>,
}

pub trait Package {
  // Metadata
  fn name(&self) -> &str;
  fn title(&self) -> &str;
  fn description(&self) -> &str;
  fn version(&self) -> &str;
  fn author(&self) -> &str;

  // Directory containing the package files
  fn location(&self) -> &Path;
}

pub trait PackageResolver {
  fn download(&self, package: &PackageSpecifier) -> Result<Box<dyn Package>>;
  // TODO: fn check update available?
  // TODO: fn update
}

// TODO: the git resolver should delete the .git directory

#[derive(Error, Debug)]
pub enum PackageResolutionError {
  #[error("package not found")]
  PackageNotFound,
}
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

use anyhow::Result;

use crate::Package;

pub(crate) mod hub;
pub(crate) mod git;
pub(crate) mod github;
pub(crate) mod gitlab;

#[derive(Debug, Default)]
pub struct PackageSpecifier {
  pub name: String,
  pub version: Option<String>,

  // Source information
  pub git_repo_url: Option<String>,
  pub git_branch: Option<String>,

  // Resolution options
  pub use_native_git: bool,
}

pub trait PackageProvider {
  fn name(&self) -> String;
  fn download(&self, package: &PackageSpecifier) -> Result<Box<dyn Package>>;
  // TODO: fn check update available? (probably should be only available in the hub)
}

#[derive(Debug, Default)]
pub struct ProviderOptions {
  pub force_index_update: bool,
}
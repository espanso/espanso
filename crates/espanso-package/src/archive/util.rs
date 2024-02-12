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
use fs_extra::dir::CopyOptions;

use crate::PackageSpecifier;

use super::{PackageSource, PACKAGE_SOURCE_FILE};

// TODO: test
pub fn copy_dir_without_dot_files(source_dir: &Path, inside_dir: &Path) -> Result<()> {
  fs_extra::dir::copy(
    source_dir,
    inside_dir,
    &CopyOptions {
      copy_inside: true,
      content_only: true,
      ..Default::default()
    },
  )?;

  // Remove dot files and dirs (such as .git)
  let mut to_be_removed = Vec::new();
  for path in std::fs::read_dir(inside_dir)? {
    let path = path?.path();
    if path.starts_with(".") {
      to_be_removed.push(path);
    }
  }

  fs_extra::remove_items(&to_be_removed)?;

  Ok(())
}

pub fn create_package_source_file(specifier: &PackageSpecifier, target_dir: &Path) -> Result<()> {
  let source: PackageSource = specifier.into();
  let yaml = serde_yaml::to_string(&source)?;
  std::fs::write(target_dir.join(PACKAGE_SOURCE_FILE), yaml)?;
  Ok(())
}

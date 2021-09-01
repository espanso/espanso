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

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

use crate::{ArchivedPackage, Archiver, Package, PackageSpecifier, SaveOptions};

use super::StoredPackage;

pub struct DefaultArchiver {
  package_dir: PathBuf,
}

impl DefaultArchiver {
  pub fn new(package_dir: &Path) -> Self {
    Self {
      package_dir: package_dir.to_owned(),
    }
  }
}

impl Archiver for DefaultArchiver {
  fn save(
    &self,
    package: &dyn Package,
    specifier: &PackageSpecifier,
    save_options: &SaveOptions,
  ) -> Result<ArchivedPackage> {
    let target_dir = self.package_dir.join(package.name());

    if target_dir.is_dir() && !save_options.overwrite_existing {
      bail!("package {} is already installed", package.name());
    }

    // Backup the previous directory if present
    let backup_dir = self.package_dir.join(&format!("{}.old", package.name()));
    let _backup_guard = if target_dir.is_dir() {
      std::fs::rename(&target_dir, &backup_dir)
        .context("unable to backup old package directory")?;

      // If the function returns due to an error, restore the previous directory
      Some(scopeguard::guard(
        (backup_dir.clone(), target_dir.clone()),
        |(backup_dir, target_dir)| {
          if backup_dir.is_dir() {
            if target_dir.is_dir() {
              std::fs::remove_dir_all(&target_dir)
                .expect("unable to remove dirty package directory");
            }

            std::fs::rename(backup_dir, target_dir).expect("unable to restore backup directory");
          }
        },
      ))
    } else {
      None
    };

    std::fs::create_dir_all(&target_dir).context("unable to create target directory")?;

    super::util::copy_dir_without_dot_files(package.location(), &target_dir)
      .context("unable to copy package files")?;

    super::util::create_package_source_file(specifier, &target_dir)
      .context("unable to create _pkgsource.yml file")?;

    // Remove backup
    if backup_dir.is_dir() {
      std::fs::remove_dir_all(backup_dir).context("unable to remove backup directory")?;
    }

    let archived_package =
      super::read::read_archived_package(&target_dir).context("unable to load archived package")?;

    Ok(archived_package)
  }

  fn get(&self, name: &str) -> Result<ArchivedPackage> {
    todo!()
  }

  fn list(&self) -> Result<Vec<StoredPackage>> {
    todo!()
  }

  fn delete(&self, name: &str) -> Result<()> {
    todo!()
  }
}

// TODO: test
// TODO: test what happens with "legacy" packages

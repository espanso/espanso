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

use crate::{
    manifest::Manifest, ArchivedPackage, Archiver, Package, PackageSpecifier, SaveOptions,
};

use super::{LegacyPackage, PackageSource, StoredPackage, PACKAGE_SOURCE_FILE};

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
        let backup_dir = self.package_dir.join(format!("{}.old", package.name()));
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

                        std::fs::rename(backup_dir, target_dir)
                            .expect("unable to restore backup directory");
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

        let archived_package = super::read::read_archived_package(&target_dir)
            .context("unable to load archived package")?;

        Ok(archived_package)
    }

    fn get(&self, name: &str) -> Result<StoredPackage> {
        let target_dir = self.package_dir.join(name);

        if !target_dir.is_dir() {
            bail!("package '{}' not found", name);
        }

        let manifest_path = target_dir.join("_manifest.yml");
        if !manifest_path.is_file() {
            return Ok(StoredPackage::Legacy(LegacyPackage {
                name: name.to_string(),
            }));
        }

        let manifest =
            Manifest::parse(&manifest_path).context("unable to parse package manifest")?;

        let source_path = target_dir.join(PACKAGE_SOURCE_FILE);
        let source =
            PackageSource::parse(&source_path).context("unable to parse package source file")?;

        Ok(StoredPackage::Modern(ArchivedPackage { manifest, source }))
    }

    fn list(&self) -> Result<Vec<StoredPackage>> {
        let mut output = Vec::new();

        for path in std::fs::read_dir(&self.package_dir)? {
            let path = path?.path();
            if !path.is_dir() {
                continue;
            }

            if let Some(package_name) = path.file_name() {
                let package_name = package_name.to_string_lossy().to_string();

                output.push(self.get(&package_name)?);
            }
        }

        Ok(output)
    }

    fn delete(&self, name: &str) -> Result<()> {
        let target_dir = self.package_dir.join(name);

        if !target_dir.is_dir() {
            bail!("package {} not found", name);
        }

        std::fs::remove_dir_all(&target_dir).context("unable to remove package directory")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, write};
    use tempdir::TempDir;

    use crate::{manifest::Manifest, package::DefaultPackage, tests::run_with_temp_dir};

    use super::*;

    fn create_fake_package(dest_dir: &Path) -> Box<dyn Package> {
        let package_dir = dest_dir.join("package1");
        create_dir_all(&package_dir).unwrap();

        write(
            package_dir.join("_manifest.yml"),
            r#"
name: "package1"
title: "Dummy package"
description: A dummy package for testing
version: 0.1.0
author: Federico Terzi
    "#,
        )
        .unwrap();

        write(
            package_dir.join("package.yml"),
            r#"
matches:
  - trigger: ":hello"
    replace: "github"name: "package1"
    "#,
        )
        .unwrap();

        write(
            package_dir.join("README.md"),
            r"
    A very dummy package
    ",
        )
        .unwrap();

        let package = DefaultPackage::new(
            Manifest::parse(&package_dir.join("_manifest.yml")).unwrap(),
            TempDir::new("fake-package").unwrap(),
            package_dir,
        );

        Box::new(package)
    }

    fn run_with_two_temp_dirs(action: impl FnOnce(&Path, &Path)) {
        run_with_temp_dir(|base| {
            let dir1 = base.join("dir1");
            let dir2 = base.join("dir2");
            create_dir_all(&dir1).unwrap();
            create_dir_all(&dir2).unwrap();
            action(&dir1, &dir2);
        });
    }

    #[test]
    fn test_package_saved_correctly() {
        run_with_two_temp_dirs(|package_dir, dest_dir| {
            let package = create_fake_package(package_dir);

            let archiver = DefaultArchiver::new(dest_dir);
            let result = archiver.save(
                &*package,
                &PackageSpecifier {
                    name: "package1".to_string(),
                    git_repo_url: Some("https://github.com/espanso/dummy-package".to_string()),
                    git_branch: Some("main".to_string()),
                    ..Default::default()
                },
                &SaveOptions::default(),
            );

            assert!(result.is_ok());

            let package_out_dir = dest_dir.join("package1");
            assert!(package_out_dir.is_dir());
            assert!(package_out_dir.join("_manifest.yml").is_file());
            assert!(package_out_dir.join("README.md").is_file());
            assert!(package_out_dir.join("package.yml").is_file());
            assert!(package_out_dir.join("_pkgsource.yml").is_file());
        });
    }

    #[test]
    fn test_package_already_present() {
        run_with_two_temp_dirs(|package_dir, dest_dir| {
            let package = create_fake_package(package_dir);

            create_dir_all(dest_dir.join("package1")).unwrap();

            let archiver = DefaultArchiver::new(dest_dir);
            let result = archiver.save(
                &*package,
                &PackageSpecifier {
                    name: "package1".to_string(),
                    git_repo_url: Some("https://github.com/espanso/dummy-package".to_string()),
                    git_branch: Some("main".to_string()),
                    ..Default::default()
                },
                &SaveOptions::default(),
            );

            assert!(result.is_err());

            let result = archiver.save(
                &*package,
                &PackageSpecifier {
                    name: "package1".to_string(),
                    git_repo_url: Some("https://github.com/espanso/dummy-package".to_string()),
                    git_branch: Some("main".to_string()),
                    ..Default::default()
                },
                &SaveOptions {
                    overwrite_existing: true,
                },
            );

            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_delete_package() {
        run_with_two_temp_dirs(|package_dir, dest_dir| {
            let package = create_fake_package(package_dir);

            let archiver = DefaultArchiver::new(dest_dir);
            let result = archiver.save(
                &*package,
                &PackageSpecifier {
                    name: "package1".to_string(),
                    git_repo_url: Some("https://github.com/espanso/dummy-package".to_string()),
                    git_branch: Some("main".to_string()),
                    ..Default::default()
                },
                &SaveOptions::default(),
            );

            assert!(result.is_ok());

            let package_out_dir = dest_dir.join("package1");
            assert!(package_out_dir.is_dir());

            archiver.delete("package1").unwrap();

            assert!(!package_out_dir.is_dir());
        });
    }

    #[test]
    fn test_list_packages() {
        run_with_two_temp_dirs(|package_dir, dest_dir| {
            let package = create_fake_package(package_dir);

            let archiver = DefaultArchiver::new(dest_dir);
            let result = archiver.save(
                &*package,
                &PackageSpecifier {
                    name: "package1".to_string(),
                    git_repo_url: Some("https://github.com/espanso/dummy-package".to_string()),
                    git_branch: Some("main".to_string()),
                    ..Default::default()
                },
                &SaveOptions::default(),
            );

            assert!(result.is_ok());

            let package_out_dir = dest_dir.join("package1");
            assert!(package_out_dir.is_dir());

            let legacy_package = dest_dir.join("z_legacypackage1");
            create_dir_all(legacy_package).unwrap();

            let package_list = archiver.list().unwrap();

            assert!(package_list.iter().any(|package| *package
                == StoredPackage::Modern(ArchivedPackage {
                    manifest: Manifest::parse(&package_out_dir.join("_manifest.yml")).unwrap(),
                    source: PackageSource::parse(&package_out_dir.join(PACKAGE_SOURCE_FILE))
                        .unwrap(),
                })));
            assert!(package_list.iter().any(|package| *package
                == StoredPackage::Legacy(LegacyPackage {
                    name: "z_legacypackage1".to_string()
                })));
        });
    }
}

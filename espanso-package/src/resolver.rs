/*
* This file is part of espanso.
*
* Copyright (C) 2019-2021 Federico Terzi
title: (), description: (), version: (), author: ()  *
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

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

use crate::manifest::Manifest;

#[derive(Debug, PartialEq, Eq)]
pub struct ResolvedPackage {
  pub manifest: Manifest,
  pub base_dir: PathBuf,
}

pub fn resolve_package(
  base_dir: &Path,
  name: &str,
  version: Option<&str>,
) -> Result<ResolvedPackage> {
  let packages = resolve_all_packages(base_dir)?;

  let mut matching_packages: Vec<ResolvedPackage> = packages
    .into_iter()
    .filter(|package| package.manifest.name == name)
    .collect();

  if matching_packages.is_empty() {
    bail!("no package found with name: {}", name);
  }

  matching_packages.sort_by(|a, b| natord::compare(&a.manifest.version, &b.manifest.version));

  let matching_package = if let Some(explicit_version) = version {
    matching_packages
      .into_iter()
      .find(|package| package.manifest.version == explicit_version)
  } else {
    matching_packages.into_iter().last()
  };

  if let Some(matching_package) = matching_package {
    Ok(matching_package)
  } else {
    bail!(
      "unable to find version: {} for package: {}",
      version.unwrap_or_default(),
      name
    );
  }
}

pub fn resolve_all_packages(base_dir: &Path) -> Result<Vec<ResolvedPackage>> {
  let manifest_files = find_all_manifests(base_dir)?;

  if manifest_files.is_empty() {
    bail!("no manifests found in base_dir");
  }

  let mut manifests = Vec::new();

  for manifest_file in manifest_files {
    let base_dir = manifest_file
      .parent()
      .ok_or_else(|| anyhow!("unable to determine base_dir from manifest path"))?
      .to_owned();
    let manifest = Manifest::parse(&manifest_file).context("manifest YAML parsing error")?;
    manifests.push(ResolvedPackage { manifest, base_dir });
  }

  Ok(manifests)
}

fn find_all_manifests(base_dir: &Path) -> Result<Vec<PathBuf>> {
  let pattern = format!("{}/{}", base_dir.to_string_lossy(), "**/_manifest.yml");

  let mut manifests = Vec::new();

  for entry in glob::glob(&pattern)? {
    let path = entry?;
    manifests.push(path);
  }

  Ok(manifests)
}

#[cfg(test)]
mod tests {
  use std::fs::create_dir_all;

  use crate::tests::run_with_temp_dir;

  use super::*;

  #[test]
  fn test_read_manifest_base_dir() {
    run_with_temp_dir(|base_dir| {
      std::fs::write(
        base_dir.join("_manifest.yml"),
        r#"
      name: package1
      title: Package 1
      author: Federico
      version: 0.1.0
      description: An awesome package 
      "#,
      )
      .unwrap();

      let packages = resolve_all_packages(base_dir).unwrap();

      assert_eq!(
        packages,
        vec![ResolvedPackage {
          manifest: Manifest {
            name: "package1".to_owned(),
            title: "Package 1".to_owned(),
            version: "0.1.0".to_owned(),
            author: "Federico".to_owned(),
            description: "An awesome package".to_owned(),
          },
          base_dir: base_dir.to_path_buf(),
        },]
      )
    });
  }

  #[test]
  fn test_read_manifests_nested_dirs() {
    run_with_temp_dir(|base_dir| {
      let sub_dir1 = base_dir.join("package1");
      let version_dir1 = sub_dir1.join("0.1.0");
      create_dir_all(&version_dir1).unwrap();

      std::fs::write(
        version_dir1.join("_manifest.yml"),
        r#"
      name: package1
      title: Package 1
      author: Federico
      version: 0.1.0
      description: An awesome package 
      "#,
      )
      .unwrap();

      let sub_dir2 = base_dir.join("package1");
      let version_dir2 = sub_dir2.join("0.1.1");
      create_dir_all(&version_dir2).unwrap();

      std::fs::write(
        version_dir2.join("_manifest.yml"),
        r#"
      name: package1
      title: Package 1
      author: Federico
      version: 0.1.1
      description: An awesome package 
      "#,
      )
      .unwrap();

      let sub_dir3 = base_dir.join("package2");
      create_dir_all(&sub_dir3).unwrap();

      std::fs::write(
        sub_dir3.join("_manifest.yml"),
        r#"
      name: package2
      title: Package 2
      author: Federico
      version: 2.0.0
      description: Another awesome package 
      "#,
      )
      .unwrap();

      let packages = resolve_all_packages(base_dir).unwrap();

      assert_eq!(
        packages,
        vec![
          ResolvedPackage {
            manifest: Manifest {
              name: "package1".to_owned(),
              title: "Package 1".to_owned(),
              version: "0.1.0".to_owned(),
              author: "Federico".to_owned(),
              description: "An awesome package".to_owned(),
            },
            base_dir: version_dir1,
          },
          ResolvedPackage {
            manifest: Manifest {
              name: "package1".to_owned(),
              title: "Package 1".to_owned(),
              version: "0.1.1".to_owned(),
              author: "Federico".to_owned(),
              description: "An awesome package".to_owned(),
            },
            base_dir: version_dir2,
          },
          ResolvedPackage {
            manifest: Manifest {
              name: "package2".to_owned(),
              title: "Package 2".to_owned(),
              version: "2.0.0".to_owned(),
              author: "Federico".to_owned(),
              description: "Another awesome package".to_owned(),
            },
            base_dir: sub_dir3,
          },
        ]
      )
    });
  }

  #[test]
  fn test_resolve_package() {
    run_with_temp_dir(|base_dir| {
      let sub_dir1 = base_dir.join("package1");
      let version_dir1 = sub_dir1.join("0.1.0");
      create_dir_all(&version_dir1).unwrap();

      std::fs::write(
        version_dir1.join("_manifest.yml"),
        r#"
      name: package1
      title: Package 1
      author: Federico
      version: 0.1.0
      description: An awesome package 
      "#,
      )
      .unwrap();

      let sub_dir2 = base_dir.join("package1");
      let version_dir2 = sub_dir2.join("0.1.1");
      create_dir_all(&version_dir2).unwrap();

      std::fs::write(
        version_dir2.join("_manifest.yml"),
        r#"
      name: package1
      title: Package 1
      author: Federico
      version: 0.1.1
      description: An awesome package 
      "#,
      )
      .unwrap();

      let sub_dir3 = base_dir.join("package2");
      create_dir_all(&sub_dir3).unwrap();

      std::fs::write(
        sub_dir3.join("_manifest.yml"),
        r#"
      name: package2
      title: Package 2
      author: Federico
      version: 2.0.0
      description: Another awesome package 
      "#,
      )
      .unwrap();

      assert_eq!(
        resolve_package(base_dir, "package1", None).unwrap(),
        ResolvedPackage {
          manifest: Manifest {
            name: "package1".to_owned(),
            title: "Package 1".to_owned(),
            version: "0.1.1".to_owned(),
            author: "Federico".to_owned(),
            description: "An awesome package".to_owned(),
          },
          base_dir: version_dir2,
        },
      );

      assert_eq!(
        resolve_package(base_dir, "package1", Some("0.1.0")).unwrap(),
        ResolvedPackage {
          manifest: Manifest {
            name: "package1".to_owned(),
            title: "Package 1".to_owned(),
            version: "0.1.0".to_owned(),
            author: "Federico".to_owned(),
            description: "An awesome package".to_owned(),
          },
          base_dir: version_dir1,
        },
      );

      assert_eq!(
        resolve_package(base_dir, "package2", None).unwrap(),
        ResolvedPackage {
          manifest: Manifest {
            name: "package2".to_owned(),
            title: "Package 2".to_owned(),
            version: "2.0.0".to_owned(),
            author: "Federico".to_owned(),
            description: "Another awesome package".to_owned(),
          },
          base_dir: sub_dir3,
        },
      );

      assert!(resolve_package(base_dir, "invalid", None).is_err());
      assert!(resolve_package(base_dir, "package1", Some("9.9.9")).is_err());
    });
  }

  #[test]
  fn test_no_manifest_error() {
    run_with_temp_dir(|base_dir| {
      assert!(resolve_all_packages(base_dir).is_err());
    });
  }

  #[test]
  fn test_malformed_manifest() {
    run_with_temp_dir(|base_dir| {
      std::fs::write(
        base_dir.join("_manifest.yml"),
        r#"
      name: package1
      title: Package 1
      author: Federico
      "#,
      )
      .unwrap();

      assert!(resolve_all_packages(base_dir).is_err());
    });
  }
}

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

use anyhow::{bail, Context, Result};

use crate::{manifest::Manifest, ArchivedPackage};

use super::{PackageSource, PACKAGE_SOURCE_FILE};

pub fn read_archived_package(containing_dir: &Path) -> Result<ArchivedPackage> {
    let manifest_path = containing_dir.join("_manifest.yml");
    if !manifest_path.is_file() {
        bail!("missing _manifest.yml file");
    }

    let source_path = containing_dir.join(PACKAGE_SOURCE_FILE);
    let source = if source_path.is_file() {
        let yaml = std::fs::read_to_string(&source_path)?;
        let source: PackageSource =
            serde_yaml::from_str(&yaml).context("unable to parse package source file.")?;
        source
    } else {
        // Fallback to hub installation
        PackageSource::Hub
    };

    let manifest = Manifest::parse(&manifest_path).context("unable to parse manifest file")?;

    Ok(ArchivedPackage { manifest, source })
}

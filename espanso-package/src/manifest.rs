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

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct Manifest {
    pub name: String,
    pub title: String,
    pub description: String,
    pub version: String,
    pub author: String,
}

impl Manifest {
    pub fn parse(manifest_path: &Path) -> Result<Self> {
        let manifest_str = std::fs::read_to_string(manifest_path)?;

        serde_yaml::from_str(&manifest_str).with_context(|| {
            format!(
                "Failed manifest parsing for path: {}",
                manifest_path.display()
            )
        })
    }
}

// TODO: test

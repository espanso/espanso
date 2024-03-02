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

use std::path::PathBuf;

use tempdir::TempDir;

use crate::{manifest::Manifest, Package};

#[allow(dead_code)]
pub struct DefaultPackage {
    manifest: Manifest,

    temp_dir: TempDir,

    // Sub-directory inside the temp_dir
    location: PathBuf,
}

impl DefaultPackage {
    pub fn new(manifest: Manifest, temp_dir: TempDir, location: PathBuf) -> Self {
        Self {
            manifest,
            temp_dir,
            location,
        }
    }
}

impl Package for DefaultPackage {
    fn name(&self) -> &str {
        self.manifest.name.as_str()
    }

    fn title(&self) -> &str {
        self.manifest.title.as_str()
    }

    fn description(&self) -> &str {
        self.manifest.description.as_str()
    }

    fn version(&self) -> &str {
        self.manifest.version.as_str()
    }

    fn author(&self) -> &str {
        self.manifest.author.as_str()
    }

    fn location(&self) -> &std::path::Path {
        self.location.as_path()
    }
}

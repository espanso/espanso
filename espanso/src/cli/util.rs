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

use super::PathsOverrides;
use std::process::Command;

pub trait CommandExt {
  fn with_paths_overrides(&mut self, paths_overrides: &PathsOverrides) -> &mut Self;
}

impl CommandExt for Command {
  fn with_paths_overrides(&mut self, paths_overrides: &PathsOverrides) -> &mut Self {
    // We only inject the paths that were explicitly overrided because otherwise
    // the migration process might create some incompatibilities.
    // For example, after the migration the "packages" dir could have been
    // moved to a different one, and that might cause the daemon to crash
    // if we inject the old packages dir that was automatically resolved.
    if let Some(config_override) = &paths_overrides.config {
      self.env(
        "ESPANSO_CONFIG_DIR",
        config_override.to_string_lossy().to_string(),
      );
    }
    if let Some(packages_override) = &paths_overrides.packages {
      self.env(
        "ESPANSO_PACKAGE_DIR",
        packages_override.to_string_lossy().to_string(),
      );
    }
    if let Some(runtime_override) = &paths_overrides.runtime {
      self.env(
        "ESPANSO_RUNTIME_DIR",
        runtime_override.to_string_lossy().to_string(),
      );
    }

    self
  }
}

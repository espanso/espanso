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

pub mod default;

pub trait Package {
  // Manifest
  fn name(&self) -> &str;
  fn title(&self) -> &str;
  fn description(&self) -> &str;
  fn version(&self) -> &str;
  fn author(&self) -> &str;

  // Directory containing the package files
  fn location(&self) -> &Path;
}

impl std::fmt::Debug for dyn Package {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(
      f,
      "name: {}\nversion: {}\ntitle: {}\ndescription: {}\nauthor: {}\nlocation: {:?}",
      self.name(),
      self.version(),
      self.title(),
      self.description(),
      self.author(),
      self.location()
    )
  }
}

pub use default::DefaultPackage;
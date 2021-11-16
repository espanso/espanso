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

use crate::gui::TextUI;

use super::manager::ModuloManager;

pub struct ModuloTextUI<'a> {
  manager: &'a ModuloManager,
}

impl<'a> ModuloTextUI<'a> {
  pub fn new(manager: &'a ModuloManager) -> Self {
    Self { manager }
  }
}

impl<'a> TextUI for ModuloTextUI<'a> {
  fn show_text(&self, title: &str, text: &str) -> anyhow::Result<()> {
    self
      .manager
      .spawn(&["textview", "--title", title, "-i", "-"], text)?;

    Ok(())
  }

  fn show_file(&self, title: &str, path: &std::path::Path) -> anyhow::Result<()> {
    let path_str = path.to_string_lossy().to_string();
    self
      .manager
      .spawn(&["textview", "--title", title, "-i", &path_str], "")?;

    Ok(())
  }
}

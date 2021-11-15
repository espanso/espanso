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

use espanso_engine::dispatch::TextUIHandler;

use crate::gui::TextUI;

pub struct TextUIHandlerAdapter<'a> {
  text_ui: &'a dyn TextUI,
}

impl<'a> TextUIHandlerAdapter<'a> {
  pub fn new(text_ui: &'a dyn TextUI) -> Self {
    Self { text_ui }
  }
}

impl<'a> TextUIHandler for TextUIHandlerAdapter<'a> {
  fn show_text(&self, title: &str, text: &str) -> anyhow::Result<()> {
    self.text_ui.show_text(title, text)?;
    Ok(())
  }
}

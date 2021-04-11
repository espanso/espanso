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

use espanso_inject::Injector;

use crate::engine::dispatch::TextInjector;

pub struct TextInjectorAdapter<'a> {
  injector: &'a dyn Injector,
}

impl <'a> TextInjectorAdapter<'a> {
  pub fn new(injector: &'a dyn Injector) -> Self {
    Self {
      injector
    }
  }
}

impl <'a> TextInjector for TextInjectorAdapter<'a> {
  fn inject_text(&self, text: &str) -> anyhow::Result<()> {
    // TODO: handle injection options
    self.injector.send_string(text, Default::default())
  }
}

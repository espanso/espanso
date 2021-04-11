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

use super::super::{Event, Executor, TextInjector};
use crate::engine::event::text::TextInjectMode;
use log::error;

pub struct TextInjectExecutor<'a> {
  injector: &'a dyn TextInjector,
}

impl<'a> TextInjectExecutor<'a> {
  pub fn new(injector: &'a dyn TextInjector) -> Self {
    Self { injector }
  }
}

impl<'a> Executor for TextInjectExecutor<'a> {
  fn execute(&self, event: &Event) -> bool {
    if let Event::TextInject(inject_event) = event {
      if let Some(TextInjectMode::Keys) = inject_event.force_mode {
        if let Err(error) = self.injector.inject_text(&inject_event.text) {
          error!("text injector reported an error: {:?}", error);
        }
        return true;
      }
    }

    false
  }
}

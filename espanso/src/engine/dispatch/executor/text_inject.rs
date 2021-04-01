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
use crate::engine::event::inject::TextInjectMode;
use log::error;

pub struct TextInjectExecutor<T: TextInjector> {
  injector: T,
}

impl<T: TextInjector> TextInjectExecutor<T> {
  pub fn new(injector: T) -> Self {
    Self { injector }
  }
}

impl<T: TextInjector> Executor for TextInjectExecutor<T> {
  fn execute(&self, event: &Event) -> bool {
    if let Event::TextInject(inject_event) = event {
      if inject_event.mode == TextInjectMode::Keys {
        if let Err(error) = self.injector.inject(&inject_event.text) {
          error!("text injector reported an error: {:?}", error);
        }
        return true;
      }
    }

    false
  }
}

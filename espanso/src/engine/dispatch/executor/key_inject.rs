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

use super::super::{Event, Executor, KeyInjector};
use log::error;

pub struct KeyInjectExecutor<'a> {
  injector: &'a dyn KeyInjector,
}

impl<'a> KeyInjectExecutor<'a> {
  pub fn new(injector: &'a dyn KeyInjector) -> Self {
    Self { injector }
  }
}

impl<'a> Executor for KeyInjectExecutor<'a> {
  fn execute(&self, event: &Event) -> bool {
    if let Event::KeySequenceInject(inject_event) = event {
      if let Err(error) = self.injector.inject_sequence(&inject_event.keys) {
        error!("key injector reported an error: {}", error);
      }
      return true;
    }

    false
  }
}

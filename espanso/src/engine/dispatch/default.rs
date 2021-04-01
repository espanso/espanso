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

use super::{Dispatcher, Executor, TextInjector};
use super::Event;

pub struct DefaultDispatcher {
  executors: Vec<Box<dyn Executor>>,
}

impl DefaultDispatcher {
  pub fn new(text_injector: impl TextInjector + 'static) -> Self {
    Self {
      executors: vec![
        Box::new(super::executor::text_inject::TextInjectExecutor::new(text_injector)),
      ]
    }
  }
}

impl Dispatcher for DefaultDispatcher {
  fn dispatch(&self, event: Event) {
    for executor in self.executors.iter() {
      if executor.execute(&event) {
        break
      }
    }
  }
}

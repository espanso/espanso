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

use anyhow::Result;
use super::Event;

mod executor;
mod default;

pub trait Executor {
  fn execute(&self, event: &Event) -> bool;
}

pub trait Dispatcher {
  fn dispatch(&self, event: Event);
}

pub trait TextInjector {
  fn inject(&self, text: &str) -> Result<()>;
}

pub fn default(text_injector: impl TextInjector + 'static) -> impl Dispatcher {
  default::DefaultDispatcher::new(
    text_injector,
  )
}
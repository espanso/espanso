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


use std::collections::HashMap;

use super::{Event, event::keyboard::Key};

mod middleware;
mod default;

pub trait Middleware {
  fn next(&self, event: Event, dispatch: &dyn FnMut(Event)) -> Event;
}

pub trait Processor {
  fn process(&mut self, event: Event) -> Vec<Event>;
}

// Dependency inversion entities

pub trait Matcher<State> {
  fn process(&self, prev_state: Option<&State>, event: &MatcherEvent) -> (State, Vec<MatchResult>);
}

pub enum MatcherEvent {
  Key { key: Key, chars: Option<String> },
  VirtualSeparator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchResult {
  id: i32,
  trigger: String,
  vars: HashMap<String, String>,
}

pub fn default<MatcherState: 'static>(matchers: Vec<Box<dyn Matcher<MatcherState>>>) -> impl Processor {
  default::DefaultProcessor::new(matchers)
}
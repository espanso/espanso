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

use super::{Event, event::{EventType, input::Key, internal::DetectedMatch}};
use anyhow::Result;
use std::collections::HashMap;
use thiserror::Error;

mod default;
mod middleware;

pub trait Middleware {
  fn name(&self) -> &'static str;
  fn next(&self, event: Event, dispatch: &mut dyn FnMut(Event)) -> Event;
}

pub trait Processor {
  fn process(&mut self, event: Event) -> Vec<Event>;
}

// Dependency inversion entities

// TODO: move these traits inside the various modules and then re-export it

pub trait Matcher<'a, State> {
  fn process(
    &'a self,
    prev_state: Option<&State>,
    event: &MatcherEvent,
  ) -> (State, Vec<MatchResult>);
}

#[derive(Debug)]
pub enum MatcherEvent {
  Key { key: Key, chars: Option<String> },
  VirtualSeparator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchResult {
  pub id: i32,
  pub trigger: String,
  pub left_separator: Option<String>,
  pub right_separator: Option<String>,
  pub args: HashMap<String, String>,
}

pub trait MatchFilter {
  fn filter_active(&self, matches_ids: &[i32]) -> Vec<i32>;
}

pub trait MatchSelector {
  fn select(&self, matches_ids: &[i32]) -> Option<i32>;
}

pub trait Multiplexer {
  fn convert(
    &self,
    m: DetectedMatch,
  ) -> Option<EventType>;
}

pub trait Renderer<'a> {
  fn render(&'a self, match_id: i32, trigger: Option<&str>, trigger_args: HashMap<String, String>) -> Result<String>;
}

#[derive(Error, Debug)]
pub enum RendererError {
  #[error("rendering error")]
  RenderingError(#[from] anyhow::Error),

  #[error("match not found")]
  NotFound,

  #[error("aborted")]
  Aborted,
}

pub use middleware::action::{MatchInfoProvider, EventSequenceProvider};
pub use middleware::delay_modifiers::ModifierStatusProvider;

pub fn default<'a, MatcherState>(
  matchers: &'a [&'a dyn Matcher<'a, MatcherState>],
  match_filter: &'a dyn MatchFilter,
  match_selector: &'a dyn MatchSelector,
  multiplexer: &'a dyn Multiplexer,
  renderer: &'a dyn Renderer<'a>,
  match_info_provider: &'a dyn MatchInfoProvider,
  modifier_status_provider: &'a dyn ModifierStatusProvider,
  event_sequence_provider: &'a dyn EventSequenceProvider,
) -> impl Processor + 'a {
  default::DefaultProcessor::new(
    matchers,
    match_filter,
    match_selector,
    multiplexer,
    renderer,
    match_info_provider,
    modifier_status_provider,
    event_sequence_provider,
  )
}

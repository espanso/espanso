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

use super::Event;

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

pub use middleware::action::{EventSequenceProvider, MatchInfoProvider};
pub use middleware::alt_code_synthesizer::AltCodeSynthEnabledProvider;
pub use middleware::delay_modifiers::ModifierStatusProvider;
pub use middleware::disable::DisableOptions;
pub use middleware::image_resolve::PathProvider;
pub use middleware::match_exec::MatchResolver;
pub use middleware::match_select::{MatchFilter, MatchSelector};
pub use middleware::matcher::{
  MatchResult, Matcher, MatcherEvent, MatcherMiddlewareConfigProvider, ModifierState,
  ModifierStateProvider,
};
pub use middleware::multiplex::Multiplexer;
pub use middleware::notification::NotificationManager;
pub use middleware::open_config::ConfigPathProvider;
pub use middleware::render::{Renderer, RendererError};
pub use middleware::search::MatchProvider;
pub use middleware::suppress::EnabledStatusProvider;
pub use middleware::undo::UndoEnabledProvider;

#[allow(clippy::too_many_arguments)]
pub fn default<'a, MatcherState>(
  matchers: &'a [&'a dyn Matcher<'a, MatcherState>],
  match_filter: &'a dyn MatchFilter,
  match_selector: &'a dyn MatchSelector,
  multiplexer: &'a dyn Multiplexer,
  renderer: &'a dyn Renderer<'a>,
  match_info_provider: &'a dyn MatchInfoProvider,
  modifier_status_provider: &'a dyn ModifierStatusProvider,
  event_sequence_provider: &'a dyn EventSequenceProvider,
  path_provider: &'a dyn PathProvider,
  config_path_provider: &'a dyn ConfigPathProvider,
  disable_options: DisableOptions,
  matcher_options_provider: &'a dyn MatcherMiddlewareConfigProvider,
  match_provider: &'a dyn MatchProvider,
  undo_enabled_provider: &'a dyn UndoEnabledProvider,
  enabled_status_provider: &'a dyn EnabledStatusProvider,
  modifier_state_provider: &'a dyn ModifierStateProvider,
  match_resolver: &'a dyn MatchResolver,
  notification_manager: &'a dyn NotificationManager,
  alt_code_synth_enabled_provider: &'a dyn AltCodeSynthEnabledProvider,
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
    path_provider,
    config_path_provider,
    disable_options,
    matcher_options_provider,
    match_provider,
    undo_enabled_provider,
    enabled_status_provider,
    modifier_state_provider,
    match_resolver,
    notification_manager,
    alt_code_synth_enabled_provider,
  )
}

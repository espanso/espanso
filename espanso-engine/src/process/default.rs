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

use log::trace;

use super::{
  middleware::{
    action::{ActionMiddleware, EventSequenceProvider},
    alt_code_synthesizer::AltCodeSynthesizerMiddleware,
    cause::CauseCompensateMiddleware,
    cursor_hint::CursorHintMiddleware,
    delay_modifiers::{DelayForModifierReleaseMiddleware, ModifierStatusProvider},
    discard::EventsDiscardMiddleware,
    markdown::MarkdownMiddleware,
    match_select::MatchSelectMiddleware,
    matcher::MatcherMiddleware,
    multiplex::MultiplexMiddleware,
    open_config::ConfigMiddleware,
    open_config::ConfigPathProvider,
    render::RenderMiddleware,
  },
  AltCodeSynthEnabledProvider, DisableOptions, EnabledStatusProvider, MatchFilter,
  MatchInfoProvider, MatchProvider, MatchResolver, MatchSelector, Matcher,
  MatcherMiddlewareConfigProvider, Middleware, ModifierStateProvider, Multiplexer,
  NotificationManager, PathProvider, Processor, Renderer, UndoEnabledProvider,
};
use crate::{
  event::{Event, EventType},
  process::middleware::{
    context_menu::ContextMenuMiddleware, disable::DisableMiddleware, exit::ExitMiddleware,
    hotkey::HotKeyMiddleware, icon_status::IconStatusMiddleware,
    image_resolve::ImageResolverMiddleware, match_exec::MatchExecRequestMiddleware,
    notification::NotificationMiddleware, search::SearchMiddleware, suppress::SuppressMiddleware,
    undo::UndoMiddleware,
  },
};
use std::collections::VecDeque;

pub struct DefaultProcessor<'a> {
  event_queue: VecDeque<Event>,
  middleware: Vec<Box<dyn Middleware + 'a>>,
}

#[allow(clippy::too_many_arguments)]
impl<'a> DefaultProcessor<'a> {
  pub fn new<MatcherState>(
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
  ) -> DefaultProcessor<'a> {
    Self {
      event_queue: VecDeque::new(),
      middleware: vec![
        Box::new(EventsDiscardMiddleware::new()),
        Box::new(DisableMiddleware::new(disable_options)),
        Box::new(IconStatusMiddleware::new()),
        Box::new(AltCodeSynthesizerMiddleware::new(
          alt_code_synth_enabled_provider,
        )),
        Box::new(MatcherMiddleware::new(
          matchers,
          matcher_options_provider,
          modifier_state_provider,
        )),
        Box::new(MatchExecRequestMiddleware::new(match_resolver)),
        Box::new(SuppressMiddleware::new(enabled_status_provider)),
        Box::new(ContextMenuMiddleware::new()),
        Box::new(HotKeyMiddleware::new()),
        Box::new(MatchSelectMiddleware::new(
          match_filter,
          match_selector,
          event_sequence_provider,
        )),
        Box::new(CauseCompensateMiddleware::new()),
        Box::new(ConfigMiddleware::new(config_path_provider)),
        Box::new(MultiplexMiddleware::new(multiplexer)),
        Box::new(RenderMiddleware::new(renderer)),
        Box::new(ImageResolverMiddleware::new(path_provider)),
        Box::new(CursorHintMiddleware::new()),
        Box::new(ExitMiddleware::new()),
        Box::new(UndoMiddleware::new(undo_enabled_provider)),
        Box::new(ActionMiddleware::new(
          match_info_provider,
          event_sequence_provider,
        )),
        Box::new(SearchMiddleware::new(match_provider)),
        Box::new(MarkdownMiddleware::new()),
        Box::new(NotificationMiddleware::new(notification_manager)),
        Box::new(DelayForModifierReleaseMiddleware::new(
          modifier_status_provider,
        )),
      ],
    }
  }

  fn process_one(&mut self) -> Option<Event> {
    if let Some(event) = self.event_queue.pop_back() {
      let mut current_event = event;

      let mut current_queue = VecDeque::new();
      let mut dispatch = |event: Event| {
        trace!("dispatched event: {:?}", event);
        current_queue.push_front(event);
      };

      trace!("--------------- new event -----------------");
      for middleware in &self.middleware {
        trace!(
          "middleware '{}' received event: {:?}",
          middleware.name(),
          current_event
        );

        current_event = middleware.next(current_event, &mut dispatch);

        trace!(
          "middleware '{}' produced event: {:?}",
          middleware.name(),
          current_event
        );

        if let EventType::NOOP = current_event.etype {
          trace!("interrupting chain as the event is NOOP");
          break;
        }
      }

      while let Some(event) = current_queue.pop_back() {
        self.event_queue.push_front(event);
      }

      Some(current_event)
    } else {
      None
    }
  }
}

impl<'a> Processor for DefaultProcessor<'a> {
  fn process(&mut self, event: Event) -> Vec<Event> {
    self.event_queue.push_front(event);

    let mut processed_events = Vec::new();

    while !self.event_queue.is_empty() {
      if let Some(event) = self.process_one() {
        processed_events.push(event);
      }
    }

    processed_events
  }
}

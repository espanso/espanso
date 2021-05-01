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

use super::{MatchFilter, MatchInfoProvider, MatchSelector, Matcher, Middleware, Multiplexer, PathProvider, Processor, Renderer, middleware::{
    match_select::MatchSelectMiddleware, matcher::MatcherMiddleware, multiplex::MultiplexMiddleware,
    render::RenderMiddleware, action::{ActionMiddleware, EventSequenceProvider}, cursor_hint::CursorHintMiddleware, cause::CauseCompensateMiddleware,
    delay_modifiers::{DelayForModifierReleaseMiddleware, ModifierStatusProvider}, markdown::MarkdownMiddleware,
    past_discard::PastEventsDiscardMiddleware,
  }};
use crate::engine::{event::{Event, EventType}, process::middleware::image_resolve::ImageResolverMiddleware};
use std::collections::VecDeque;

pub struct DefaultProcessor<'a> {
  event_queue: VecDeque<Event>,
  middleware: Vec<Box<dyn Middleware + 'a>>,
}

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
  ) -> DefaultProcessor<'a> {
    Self {
      event_queue: VecDeque::new(),
      middleware: vec![
        Box::new(PastEventsDiscardMiddleware::new()),
        Box::new(MatcherMiddleware::new(matchers)),
        Box::new(MatchSelectMiddleware::new(match_filter, match_selector)),
        Box::new(CauseCompensateMiddleware::new()),
        Box::new(MultiplexMiddleware::new(multiplexer)),
        Box::new(RenderMiddleware::new(renderer)),
        Box::new(ImageResolverMiddleware::new(path_provider)),
        Box::new(CursorHintMiddleware::new()),
        Box::new(ActionMiddleware::new(match_info_provider, event_sequence_provider)),
        Box::new(MarkdownMiddleware::new()),
        Box::new(DelayForModifierReleaseMiddleware::new(modifier_status_provider)),
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
      for middleware in self.middleware.iter() {
        trace!("middleware '{}' received event: {:?}", middleware.name(), current_event);

        current_event = middleware.next(current_event, &mut dispatch);

        trace!("middleware '{}' produced event: {:?}", middleware.name(), current_event);

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

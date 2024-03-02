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

use super::super::Middleware;
use crate::event::{
  effect::{
    HtmlInjectRequest, ImageInjectRequest, KeySequenceInjectRequest, MarkdownInjectRequest,
    TextInjectMode, TextInjectRequest,
  },
  input::Key,
  internal::{DiscardPreviousEvent, TextFormat},
  Event, EventType,
};

pub trait MatchInfoProvider {
  fn get_force_mode(&self, match_id: i32) -> Option<TextInjectMode>;
}

pub trait EventSequenceProvider {
  fn get_next_id(&self) -> u32;
}

pub struct ActionMiddleware<'a> {
  match_info_provider: &'a dyn MatchInfoProvider,
  event_sequence_provider: &'a dyn EventSequenceProvider,
}

impl<'a> ActionMiddleware<'a> {
  pub fn new(
    match_info_provider: &'a dyn MatchInfoProvider,
    event_sequence_provider: &'a dyn EventSequenceProvider,
  ) -> Self {
    Self {
      match_info_provider,
      event_sequence_provider,
    }
  }
}

impl<'a> Middleware for ActionMiddleware<'a> {
  fn name(&self) -> &'static str {
    "action"
  }

  fn next(&self, event: Event, dispatch: &mut dyn FnMut(Event)) -> Event {
    match &event.etype {
      EventType::Rendered(_) | EventType::ImageResolved(_) => {
        dispatch(Event::caused_by(event.source_id, EventType::MatchInjected));
        dispatch(Event::caused_by(
          event.source_id,
          EventType::DiscardPrevious(DiscardPreviousEvent {
            minimum_source_id: self.event_sequence_provider.get_next_id(),
          }),
        ));

        match &event.etype {
          EventType::Rendered(m_event) => Event::caused_by(
            event.source_id,
            match m_event.format {
              TextFormat::Plain => EventType::TextInject(TextInjectRequest {
                text: m_event.body.clone(),
                force_mode: self.match_info_provider.get_force_mode(m_event.match_id),
              }),
              TextFormat::Html => EventType::HtmlInject(HtmlInjectRequest {
                html: m_event.body.clone(),
              }),
              TextFormat::Markdown => EventType::MarkdownInject(MarkdownInjectRequest {
                markdown: m_event.body.clone(),
              }),
            },
          ),
          EventType::ImageResolved(m_event) => Event::caused_by(
            event.source_id,
            EventType::ImageInject(ImageInjectRequest {
              image_path: m_event.image_path.clone(),
            }),
          ),
          _ => unreachable!(),
        }
      }
      EventType::CursorHintCompensation(m_event) => {
        dispatch(Event::caused_by(
          event.source_id,
          EventType::DiscardPrevious(DiscardPreviousEvent {
            minimum_source_id: self.event_sequence_provider.get_next_id(),
          }),
        ));

        Event::caused_by(
          event.source_id,
          EventType::KeySequenceInject(KeySequenceInjectRequest {
            keys: (0..m_event.cursor_hint_back_count)
              .map(|_| Key::ArrowLeft)
              .collect(),
          }),
        )
      }
      EventType::TriggerCompensation(m_event) => {
        let mut backspace_count = m_event.trigger.chars().count();

        // We want to preserve the left separator if present
        if let Some(left_separator) = &m_event.left_separator {
          backspace_count -= left_separator.chars().count();
        }

        Event::caused_by(
          event.source_id,
          EventType::KeySequenceInject(KeySequenceInjectRequest {
            keys: (0..backspace_count).map(|_| Key::Backspace).collect(),
          }),
        )
      }
      EventType::Undo(m_event) => {
        // We subtract one, because the backspace that triggered the undo feature
        // already removed the last char
        let backspace_count = m_event.replace.chars().count() - 1;

        dispatch(Event::caused_by(
          event.source_id,
          EventType::TextInject(TextInjectRequest {
            text: m_event.trigger.clone(),
            force_mode: self.match_info_provider.get_force_mode(m_event.match_id),
          }),
        ));

        Event::caused_by(
          event.source_id,
          EventType::KeySequenceInject(KeySequenceInjectRequest {
            keys: (0..backspace_count).map(|_| Key::Backspace).collect(),
          }),
        )
      }
      _ => event,
    }
  }
}

// TODO: test

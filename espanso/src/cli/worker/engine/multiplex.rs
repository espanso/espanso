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

use espanso_config::matches::{Match, MatchEffect};

use crate::engine::{
  event::{
    internal::DetectedMatch,
    internal::{RenderingRequestedEvent, TextFormat},
    EventType,
  },
  process::Multiplexer,
};

pub trait MatchProvider<'a> {
  fn get(&self, match_id: i32) -> Option<&'a Match>;
}

pub struct MultiplexAdapter<'a> {
  provider: &'a dyn MatchProvider<'a>,
}

impl<'a> MultiplexAdapter<'a> {
  pub fn new(provider: &'a dyn MatchProvider<'a>) -> Self {
    Self { provider }
  }
}

impl<'a> Multiplexer for MultiplexAdapter<'a> {
  fn convert(&self, detected_match: DetectedMatch) -> Option<EventType> {
    let m = self.provider.get(detected_match.id)?;

    match &m.effect {
      MatchEffect::Text(effect) => Some(EventType::RenderingRequested(RenderingRequestedEvent {
        match_id: detected_match.id,
        trigger: detected_match.trigger,
        left_separator: detected_match.left_separator,
        right_separator: detected_match.right_separator,
        trigger_args: detected_match.args,
        format: convert_format(&effect.format),
      })),
      // TODO: think about image
      MatchEffect::None => None,
    }
  }
}

fn convert_format(format: &espanso_config::matches::TextFormat) -> TextFormat {
  match format {
    espanso_config::matches::TextFormat::Plain => TextFormat::Plain,
    espanso_config::matches::TextFormat::Markdown => TextFormat::Markdown, 
    espanso_config::matches::TextFormat::Html => TextFormat::Html, 
  }
}

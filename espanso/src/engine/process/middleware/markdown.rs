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
use crate::engine::event::{Event, EventType, effect::{HtmlInjectRequest}};

// Convert markdown injection requests to HTML on the fly
pub struct MarkdownMiddleware {}

impl MarkdownMiddleware {
  pub fn new() -> Self {
    Self {}
  }
}

impl Middleware for MarkdownMiddleware {
  fn name(&self) -> &'static str {
    "markdown"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    if let EventType::MarkdownInject(m_event) = &event.etype {
      // Render the markdown into HTML
      let html = markdown::to_html(&m_event.markdown);
      let mut html = html.trim();

      // Remove the surrounding paragraph
      if html.starts_with("<p>") {
        html = html.trim_start_matches("<p>");
      }
      if html.ends_with("</p>") {
        html = html.trim_end_matches("</p>");
      }

      return Event::caused_by(event.source_id, EventType::HtmlInject(HtmlInjectRequest {
        html: html.to_owned(),
      }))
    }

    event
  }
}

// TODO: test

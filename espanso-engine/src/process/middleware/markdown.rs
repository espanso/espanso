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

use log::error;

use super::super::Middleware;
use crate::event::{effect::HtmlInjectRequest, Event, EventType};

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
      // NOTE: we wrap the `to_html` call between catch_unwind because if the markdown is malformed,
      // the library panics. Ideally, the library would return a Result::Err in that case, but
      // for now it doesn't, so we employ that workaround.
      // See also: https://github.com/federico-terzi/espanso/issues/759
      let html = std::panic::catch_unwind(|| markdown::to_html(&m_event.markdown));
      if let Ok(html) = html {
        let mut html = html.trim();

        // Remove the surrounding paragraph
        if html.starts_with("<p>") {
          html = html.trim_start_matches("<p>");
        }
        if html.ends_with("</p>") {
          html = html.trim_end_matches("</p>");
        }

        return Event::caused_by(
          event.source_id,
          EventType::HtmlInject(HtmlInjectRequest {
            html: html.to_owned(),
          }),
        );
      } else {
        error!("unable to convert markdown to HTML, is it malformed?");

        return Event::caused_by(event.source_id, EventType::NOOP);
      }
    }

    event
  }
}

// TODO: test

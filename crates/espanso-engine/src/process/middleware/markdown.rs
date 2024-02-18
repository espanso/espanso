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
      // See also: https://github.com/espanso/espanso/issues/759
      let html = std::panic::catch_unwind(|| markdown::to_html(&m_event.markdown));
      if let Ok(html) = html {
        let html = html.trim();
        let html = remove_paragraph_tag_if_single_occurrence(html);

        return Event::caused_by(
          event.source_id,
          EventType::HtmlInject(HtmlInjectRequest {
            html: html.to_owned(),
          }),
        );
      }
      error!("unable to convert markdown to HTML, is it malformed?");
      return Event::caused_by(event.source_id, EventType::NOOP);
    }

    event
  }
}

// If the match is composed of a single paragraph, we remove the tag to avoid
// a forced "newline" on some editors. In other words, we assume that if the snippet
// is composed of a single paragraph, then it should be inlined.
// On the other hand, if the snippet is composed of multiple paragraphs, then we
// avoid removing the paragraph to prevent HTML corruption.
// See: https://github.com/espanso/espanso/issues/811
fn remove_paragraph_tag_if_single_occurrence(html: &str) -> &str {
  let paragraph_count = html.matches("<p>").count();
  if paragraph_count <= 1 {
    let mut new_html = html;
    if new_html.starts_with("<p>") {
      new_html = new_html.trim_start_matches("<p>");
    }
    if new_html.ends_with("</p>") {
      new_html = new_html.trim_end_matches("</p>");
    }

    new_html
  } else {
    html
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_remove_paragraph_tag_if_single_occurrence() {
    assert_eq!(
      remove_paragraph_tag_if_single_occurrence("<p>single occurrence</p>"),
      "single occurrence"
    );
    assert_eq!(
      remove_paragraph_tag_if_single_occurrence("<p>multi</p> <p>occurrence</p>"),
      "<p>multi</p> <p>occurrence</p>"
    );
  }
}

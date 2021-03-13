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

use crate::Event;

// TODO: test
pub(crate) fn extract_string_from_events(events: &[Event]) -> String {
  let mut string = String::new();

  for event in events {
    if let Event::Key { key: _, chars } = event {
      if let Some(chars) = chars {
        string.push_str(chars);
      }
    }
  }

  string
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::event::Key;

  #[test]
  fn extract_string_from_events_all_chars() {
    assert_eq!(
      extract_string_from_events(&[
        Event::Key {
          key: Key::Other,
          chars: Some("h".to_string())
        },
        Event::Key {
          key: Key::Other,
          chars: Some("e".to_string())
        },
        Event::Key {
          key: Key::Other,
          chars: Some("l".to_string())
        },
        Event::Key {
          key: Key::Other,
          chars: Some("l".to_string())
        },
        Event::Key {
          key: Key::Other,
          chars: Some("o".to_string())
        },
      ]),
      "hello"
    );
  }

  #[test]
  fn extract_string_from_events_no_chars() {
    assert_eq!(
      extract_string_from_events(&[
        Event::Key {
          key: Key::ArrowUp,
          chars: None
        },
        Event::Key {
          key: Key::ArrowUp,
          chars: None
        },
        Event::Key {
          key: Key::ArrowUp,
          chars: None
        },
      ]),
      ""
    );
  }

  #[test]
  fn extract_string_from_events_mixed() {
    assert_eq!(
      extract_string_from_events(&[
        Event::Key {
          key: Key::Other,
          chars: Some("h".to_string())
        },
        Event::Key {
          key: Key::Other,
          chars: Some("e".to_string())
        },
        Event::Key {
          key: Key::Other,
          chars: Some("l".to_string())
        },
        Event::Key {
          key: Key::Other,
          chars: Some("l".to_string())
        },
        Event::Key {
          key: Key::Other,
          chars: Some("o".to_string())
        },
        Event::Key {
          key: Key::ArrowUp,
          chars: None
        },
      ]),
      "hello"
    );
  }
}

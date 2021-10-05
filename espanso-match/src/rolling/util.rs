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

use super::matcher::IsWordSeparator;

pub(crate) fn extract_string_from_events(
  events: &[(Event, IsWordSeparator)],
) -> (String, Option<String>, Option<String>) {
  let mut string = String::new();

  let mut left_separator = None;
  let mut right_separator = None;

  for (i, (event, is_word_separator)) in events.iter().enumerate() {
    if let Event::Key {
      key: _,
      chars: Some(chars),
    } = event
    {
      string.push_str(chars);

      if *is_word_separator {
        if i == 0 {
          left_separator = Some(chars.clone());
        } else if i == (events.len() - 1) {
          right_separator = Some(chars.clone());
        }
      }
    }
  }

  (string, left_separator, right_separator)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::event::Key;

  #[test]
  fn extract_string_from_events_all_chars() {
    assert_eq!(
      extract_string_from_events(&[
        (
          Event::Key {
            key: Key::Other,
            chars: Some("h".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some("e".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some("l".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some("l".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some("o".to_string())
          },
          false
        ),
      ]),
      ("hello".to_string(), None, None)
    );
  }

  #[test]
  fn extract_string_from_events_word_separators() {
    assert_eq!(
      extract_string_from_events(&[
        (
          Event::Key {
            key: Key::Other,
            chars: Some(".".to_string())
          },
          true
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some("h".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some("i".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some(",".to_string())
          },
          true
        ),
      ]),
      (
        ".hi,".to_string(),
        Some(".".to_string()),
        Some(",".to_string())
      ),
    );
  }

  #[test]
  fn extract_string_from_events_no_chars() {
    assert_eq!(
      extract_string_from_events(&[
        (
          Event::Key {
            key: Key::ArrowUp,
            chars: None
          },
          false
        ),
        (
          Event::Key {
            key: Key::ArrowUp,
            chars: None
          },
          false
        ),
        (
          Event::Key {
            key: Key::ArrowUp,
            chars: None
          },
          false
        ),
      ]),
      ("".to_string(), None, None)
    );
  }

  #[test]
  fn extract_string_from_events_mixed() {
    assert_eq!(
      extract_string_from_events(&[
        (
          Event::Key {
            key: Key::Other,
            chars: Some("h".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some("e".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some("l".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some("l".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::Other,
            chars: Some("o".to_string())
          },
          false
        ),
        (
          Event::Key {
            key: Key::ArrowUp,
            chars: None
          },
          false
        ),
      ]),
      ("hello".to_string(), None, None)
    );
  }
}

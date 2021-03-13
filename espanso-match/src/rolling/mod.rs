/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-202 case_insensitive: (), preserve_case_markers: (), left_word: (), right_word: ()1 Federico Terzi
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

use crate::event::Key;

pub mod matcher;
mod tree;
mod util;

#[derive(Debug, Clone, PartialEq)]
pub enum RollingItem {
  WordSeparator,
  Key(Key),
  Char(String),
  CharInsensitive(String),
}

#[derive(Debug, PartialEq)]
pub struct RollingMatch<Id> {
  pub id: Id,
  pub items: Vec<RollingItem>,
}

impl<Id> RollingMatch<Id> {
  pub fn new(id: Id, items: Vec<RollingItem>) -> Self {
    Self { id, items }
  }

  pub fn from_string(id: Id, string: &str, opt: &StringMatchOptions) -> Self {
    let mut items = Vec::new();

    if opt.left_word {
      items.push(RollingItem::WordSeparator);
    }

    for (index, c) in string.chars().enumerate() {
      if opt.case_insensitive {
        // If preserve case markers is true, we want to keep the first two chars
        // as case-sensitive. This is needed to implement the "propagate_case" option.
        if opt.preserve_case_markers && index < 2 {
          items.push(RollingItem::Char(c.to_string()))
        } else {
          items.push(RollingItem::CharInsensitive(c.to_string()))
        }
      } else {
        items.push(RollingItem::Char(c.to_string()))
      }
    }

    if opt.right_word {
      items.push(RollingItem::WordSeparator);
    }

    Self { id, items }
  }

  pub fn from_items(id: Id, items: &[RollingItem]) -> Self {
    Self {
      id,
      items: items.to_vec(),
    }
  }
}

pub struct StringMatchOptions {
  case_insensitive: bool,
  preserve_case_markers: bool,
  left_word: bool,
  right_word: bool,
}

impl Default for StringMatchOptions {
  fn default() -> Self {
    Self {
      case_insensitive: false,
      preserve_case_markers: false,
      left_word: false,
      right_word: false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_match_from_string_base_case() {
    assert_eq!(
      RollingMatch::from_string(1, "test", &StringMatchOptions::default()),
      RollingMatch {
        id: 1,
        items: vec![
          RollingItem::Char("t".to_string()),
          RollingItem::Char("e".to_string()),
          RollingItem::Char("s".to_string()),
          RollingItem::Char("t".to_string()),
        ]
      }
    )
  }

  #[test]
  fn test_match_from_string_left_word() {
    assert_eq!(
      RollingMatch::from_string(1, "test", &StringMatchOptions {
        left_word: true,
        ..Default::default()
      }),
      RollingMatch {
        id: 1,
        items: vec![
          RollingItem::WordSeparator,
          RollingItem::Char("t".to_string()),
          RollingItem::Char("e".to_string()),
          RollingItem::Char("s".to_string()),
          RollingItem::Char("t".to_string()),
        ]
      }
    )
  }

  #[test]
  fn test_match_from_string_right_word() {
    assert_eq!(
      RollingMatch::from_string(1, "test", &StringMatchOptions {
        right_word: true,
        ..Default::default()
      }),
      RollingMatch {
        id: 1,
        items: vec![
          RollingItem::Char("t".to_string()),
          RollingItem::Char("e".to_string()),
          RollingItem::Char("s".to_string()),
          RollingItem::Char("t".to_string()),
          RollingItem::WordSeparator,
        ]
      }
    )
  }

  #[test]
  fn test_match_from_string_case_insensitive() {
    assert_eq!(
      RollingMatch::from_string(1, "test", &StringMatchOptions {
        case_insensitive: true,
        ..Default::default()
      }),
      RollingMatch {
        id: 1,
        items: vec![
          RollingItem::CharInsensitive("t".to_string()),
          RollingItem::CharInsensitive("e".to_string()),
          RollingItem::CharInsensitive("s".to_string()),
          RollingItem::CharInsensitive("t".to_string()),
        ]
      }
    )
  }

  #[test]
  fn test_match_from_string_preserve_case_markers() {
    assert_eq!(
      RollingMatch::from_string(1, "test", &StringMatchOptions {
        case_insensitive: true,
        preserve_case_markers: true,
        ..Default::default()
      }),
      RollingMatch {
        id: 1,
        items: vec![
          RollingItem::Char("t".to_string()),
          RollingItem::Char("e".to_string()),
          RollingItem::CharInsensitive("s".to_string()),
          RollingItem::CharInsensitive("t".to_string()),
        ]
      }
    )
  }
}

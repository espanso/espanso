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

use unicase::UniCase;

use crate::event::{Event, Key};

use super::tree::{MatcherTreeNode, MatcherTreeRef};

pub struct RollingMatcher {
  char_word_separators: Vec<String>,
  key_word_separators: Vec<Key>,

  root: MatcherTreeNode,
}

// impl Matcher for RollingMatcher {
//   fn process(
//     &self,
//     prev_state: &dyn std::any::Any,
//     event: Option<bool>,
//   ) -> (Box<dyn std::any::Any>, Vec<i32>) {
//     todo!()
//   }
// }

impl RollingMatcher {
  // TODO: to find the matches, we first call the `find_refs` to get the list of matching nodes
  // then we scan them and if any of those references is of variant `Matches`, then we return those
  // match ids, otherwise None

  // TODO: test
  fn find_refs<'a>(&'a self, node: &'a MatcherTreeNode, event: &Event) -> Vec<&'a MatcherTreeRef> {
    let mut refs = Vec::new();

    if let Event::Key { key, char } = event {
      // Key matching
      if let Some((_, node_ref)) = node.keys.iter().find(|(_key, _)| _key == key) {
        refs.push(node_ref);
      }

      if let Some(char) = char {
        // Char matching
        if let Some((_, node_ref)) = node.chars.iter().find(|(_char, _)| _char == char) {
          refs.push(node_ref);
        }

        // Char case-insensitive
        let insensitive_char = UniCase::new(char);
        if let Some((_, node_ref)) = node
          .chars_insensitive
          .iter()
          .find(|(_char, _)| *_char == insensitive_char)
        {
          refs.push(node_ref);
        }
      }
    }

    if self.is_word_separator(event) {
      if let Some(node_ref) = node.word_separators.as_ref() {
        refs.push(node_ref)
      }
    }

    refs
  }

  fn is_word_separator(&self, event: &Event) -> bool {
    match event {
      Event::Key { key, char } => {
        if self.key_word_separators.contains(&key) {
          true
        } else if let Some(char) = char {
          self.char_word_separators.contains(&char)
        } else {
          false
        }
      }
      Event::VirtualSeparator => true,
      _ => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test() {
    
  }
}

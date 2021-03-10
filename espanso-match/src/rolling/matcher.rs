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
use crate::Matcher;
use crate::event::{Event, Key};
use super::{RollingItem, RollingMatch, tree::{MatcherTreeNode, MatcherTreeRef}};

pub struct RollingMatcherState<'a, Id> {
  nodes: Vec<&'a MatcherTreeNode<Id>>,
}

impl <'a, Id> Default for RollingMatcherState<'a, Id> {
  fn default() -> Self {
    Self {
      nodes: Vec::new(),
    }
  }
}

pub struct RollingMatcher<Id> {
  char_word_separators: Vec<String>,
  key_word_separators: Vec<Key>,

  root: MatcherTreeNode<Id>,
}


impl <'a, Id> Matcher<'a, RollingMatcherState<'a, Id>, Id> for RollingMatcher<Id> where Id: Clone {
  fn process(
    &'a self,
    prev_state: Option<&'a RollingMatcherState<'a, Id>>,
    event: Event,
  ) -> (RollingMatcherState<'a, Id>, Vec<Id>) {
    let mut next_refs = Vec::new();

    // First compute the old refs
    if let Some(prev_state) = prev_state {
      for node_ref in prev_state.nodes.iter() {
        next_refs.extend(self.find_refs(node_ref, &event));
      }
    }

    // Calculate new ones
    let root_refs = self.find_refs(&self.root, &event);
    next_refs.extend(root_refs);

    let mut next_nodes = Vec::new();

    for node_ref in next_refs {
      match node_ref {
        MatcherTreeRef::Matches(matches) => {
          // Reset the state and return the matches
          return (RollingMatcherState::default(), matches.clone());
        },
        MatcherTreeRef::Node(node) => {
          next_nodes.push(node.as_ref());
        }
      }
    }

    let current_state = RollingMatcherState {
      nodes: next_nodes,
    };

    (current_state, Vec::new())
  }
}

impl <Id> RollingMatcher<Id> {
  pub fn new(matches: &[RollingMatch<Id>]) -> Self {
    todo!()
    // Self {

    // }
  }

  // TODO: test
  fn find_refs<'a>(&'a self, node: &'a MatcherTreeNode<Id>, event: &Event) -> Vec<&'a MatcherTreeRef<Id>> {
    let mut refs = Vec::new();

    if let Event::Key { key, chars } = event {
      // Key matching
      if let Some((_, node_ref)) = node.keys.iter().find(|(_key, _)| _key == key) {
        refs.push(node_ref);
      }

      if let Some(char) = chars {
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
      Event::Key { key, chars } => {
        if self.key_word_separators.contains(&key) {
          true
        } else if let Some(char) = chars {
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
  fn test() { // TODO: convert to actual test
    let root = MatcherTreeNode {
      chars: vec![
        ("h".to_string(), MatcherTreeRef::Node(Box::new(MatcherTreeNode {
            chars: vec![
              ("i".to_string(), MatcherTreeRef::Matches(vec![1])),
            ],
            ..Default::default()
          }))
        )
      ],
      ..Default::default()
    };

    let matcher = RollingMatcher {
      char_word_separators: vec![".".to_owned()],
      key_word_separators: vec![Key::ArrowUp],
      root,
    };

    let (state, matches) = matcher.process(None, Event::Key {
      key: Key::Other,
      chars: Some("h".to_string()),
    });
    assert_eq!(matches.len(), 0);

    let (state, matches) = matcher.process(Some(&state), Event::Key {
      key: Key::Other,
      chars: Some("i".to_string()),
    });
    assert_eq!(matches, vec![1]);
  }
}

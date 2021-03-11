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

use super::{
  tree::{MatcherTreeNode, MatcherTreeRef},
  RollingMatch,
};
use crate::event::{Event, Key};
use crate::Matcher;
use unicase::UniCase;

#[derive(Clone)]
pub struct RollingMatcherState<'a, Id> {
  nodes: Vec<&'a MatcherTreeNode<Id>>,
}

impl<'a, Id> Default for RollingMatcherState<'a, Id> {
  fn default() -> Self {
    Self { nodes: Vec::new() }
  }
}

pub struct RollingMatcherOptions {
  char_word_separators: Vec<String>,
  key_word_separators: Vec<Key>,
}

impl Default for RollingMatcherOptions {
  fn default() -> Self {
    Self {
      char_word_separators: Vec::new(),
      key_word_separators: Vec::new(),
    }
  }
}

pub struct RollingMatcher<Id> {
  char_word_separators: Vec<String>,
  key_word_separators: Vec<Key>,

  root: MatcherTreeNode<Id>,
}

impl<'a, Id> Matcher<'a, RollingMatcherState<'a, Id>, Id> for RollingMatcher<Id>
where
  Id: Clone,
{
  fn process(
    &'a self,
    prev_state: Option<&RollingMatcherState<'a, Id>>,
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
        }
        MatcherTreeRef::Node(node) => {
          next_nodes.push(node.as_ref());
        }
      }
    }

    let current_state = RollingMatcherState { nodes: next_nodes };

    (current_state, Vec::new())
  }
}

impl<Id: Clone> RollingMatcher<Id> {
  pub fn new(matches: &[RollingMatch<Id>], opt: RollingMatcherOptions) -> Self {
    let root = MatcherTreeNode::from_matches(matches);
    Self {
      root,
      char_word_separators: opt.char_word_separators,
      key_word_separators: opt.key_word_separators,
    }
  }

  fn find_refs<'a>(
    &'a self,
    node: &'a MatcherTreeNode<Id>,
    event: &Event,
  ) -> Vec<&'a MatcherTreeRef<Id>> {
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
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::rolling::{StringMatchOptions};

  fn get_matches_after_str<Id: Clone>(string: &str, matcher: &RollingMatcher<Id>) -> Vec<Id> {
    let mut prev_state = None;
    let mut matches = Vec::new();

    for c in string.chars() {
      let (state, _matches) = matcher.process(
        prev_state.as_ref(),
        Event::Key {
          key: Key::Other,
          chars: Some(c.to_string()),
        },
      );

      prev_state = Some(state);
      matches = _matches;
    }

    matches
  }

  #[test]
  fn matcher_process_simple_strings() {
    let matcher = RollingMatcher::new(
      &[
        RollingMatch::from_string(1, "hi", &StringMatchOptions::default()),
        RollingMatch::from_string(2, "hey", &StringMatchOptions::default()),
        RollingMatch::from_string(3, "my", &StringMatchOptions::default()),
        RollingMatch::from_string(4, "myself", &StringMatchOptions::default()),
        RollingMatch::from_string(5, "hi", &StringMatchOptions::default()),
      ],
      RollingMatcherOptions {
        ..Default::default()
      },
    );

    assert_eq!(get_matches_after_str("hi", &matcher), vec![1, 5]);
    assert_eq!(get_matches_after_str("my", &matcher), vec![3]);
    assert_eq!(get_matches_after_str("invalid", &matcher), vec![]);
  }

  #[test]
  fn matcher_process_word_matches() {
    let matcher = RollingMatcher::new(
      &[
        RollingMatch::from_string(
          1,
          "hi",
          &StringMatchOptions {
            left_word: true,
            right_word: true,
            ..Default::default()
          },
        ),
        RollingMatch::from_string(2, "hey", &StringMatchOptions::default()),
      ],
      RollingMatcherOptions {
        char_word_separators: vec![".".to_string(), ",".to_string()],
        ..Default::default()
      },
    );

    assert_eq!(get_matches_after_str("hi", &matcher), vec![]);
    assert_eq!(get_matches_after_str(".hi,", &matcher), vec![1]);
  }

  #[test]
  fn matcher_process_case_insensitive() {
    let matcher = RollingMatcher::new(
      &[
        RollingMatch::from_string(
          1,
          "hi",
          &StringMatchOptions {
            case_insensitive: true,
            ..Default::default()
          },
        ),
        RollingMatch::from_string(2, "hey", &StringMatchOptions::default()),
        RollingMatch::from_string(
          3,
          "arty",
          &StringMatchOptions {
            case_insensitive: true,
            preserve_case_markers: true,
            ..Default::default()
          },
        ),
      ],
      RollingMatcherOptions {
        char_word_separators: vec![".".to_string(), ",".to_string()],
        ..Default::default()
      },
    );

    assert_eq!(get_matches_after_str("hi", &matcher), vec![1]);
    assert_eq!(get_matches_after_str("Hi", &matcher), vec![1]);
    assert_eq!(get_matches_after_str("HI", &matcher), vec![1]);
    assert_eq!(get_matches_after_str("arty", &matcher), vec![3]);
    assert_eq!(get_matches_after_str("arTY", &matcher), vec![3]);
    assert_eq!(get_matches_after_str("ARTY", &matcher), vec![]);
  }
}

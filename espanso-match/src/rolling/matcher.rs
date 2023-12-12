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

use std::collections::HashMap;

use super::{
  tree::{MatcherTreeNode, MatcherTreeRef},
  util::extract_string_from_events,
  RollingMatch,
};
use crate::Matcher;
use crate::{
  event::{Event, Key},
  MatchResult,
};
use unicase::UniCase;

pub(crate) type IsWordSeparator = bool;

#[derive(Clone)]
pub struct RollingMatcherState<'a, Id> {
  paths: Vec<RollingMatcherStatePath<'a, Id>>,
}

impl<'a, Id> Default for RollingMatcherState<'a, Id> {
  fn default() -> Self {
    Self { paths: Vec::new() }
  }
}

#[derive(Clone)]
struct RollingMatcherStatePath<'a, Id> {
  node: &'a MatcherTreeNode<Id>,
  events: Vec<(Event, IsWordSeparator)>,
}

#[derive(Default)]
pub struct RollingMatcherOptions {
  pub char_word_separators: Vec<String>,
  pub key_word_separators: Vec<Key>,
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
  ) -> (RollingMatcherState<'a, Id>, Vec<MatchResult<Id>>) {
    let mut next_refs = Vec::new();

    // First compute the old refs
    if let Some(prev_state) = prev_state {
      for node_path in prev_state.paths.iter() {
        next_refs.extend(
          self
            .find_refs(node_path.node, &event, true)
            .into_iter()
            .map(|(node_ref, is_word_separator)| {
              let mut new_events = node_path.events.clone();
              new_events.push((event.clone(), is_word_separator));
              (node_ref, new_events)
            }),
        );
      }
    }

    // Calculate new ones
    let root_refs = self.find_refs(&self.root, &event, prev_state.is_some());
    next_refs.extend(
      root_refs
        .into_iter()
        .map(|(node_ref, is_word_separator)| (node_ref, vec![(event.clone(), is_word_separator)])),
    );

    let mut next_paths = Vec::new();

    for (node_ref, events) in next_refs {
      match node_ref {
        MatcherTreeRef::Matches(matches) => {
          let (trigger, left_separator, right_separator) = extract_string_from_events(&events);
          let results = matches
            .iter()
            .map(|id| MatchResult {
              id: id.clone(),
              trigger: trigger.clone(),
              left_separator: left_separator.clone(),
              right_separator: right_separator.clone(),
              vars: HashMap::new(),
            })
            .collect();

          // Reset the state and return the matches
          return (RollingMatcherState::default(), results);
        }
        MatcherTreeRef::Node(node) => {
          next_paths.push(RollingMatcherStatePath {
            node: node.as_ref(),
            events,
          });
        }
      }
    }

    let current_state = RollingMatcherState { paths: next_paths };

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
    has_previous_state: bool,
  ) -> Vec<(&'a MatcherTreeRef<Id>, IsWordSeparator)> {
    let mut refs = Vec::new();

    if let Event::Key { key, chars } = event {
      // Key matching
      if let Some((_, node_ref)) = node.keys.iter().find(|(k, _)| k == key) {
        refs.push((node_ref, false));
      }

      if let Some(char) = chars {
        // Char matching
        if let Some((_, node_ref)) = node.chars.iter().find(|(c, _)| c == char) {
          refs.push((node_ref, false));
        }

        // Char case-insensitive
        let insensitive_char = UniCase::new(char);
        if let Some((_, node_ref)) = node
          .chars_insensitive
          .iter()
          .find(|(c, _)| *c == insensitive_char)
        {
          refs.push((node_ref, false));
        }
      }
    }

    if self.is_word_separator(event) {
      if let Some(node_ref) = node.word_separators.as_ref() {
        refs.push((node_ref, true));
      }
    }

    // If there is no previous state, we handle it as a word separator, exploring a step forward
    // in the state.
    if !has_previous_state {
      if let Some(MatcherTreeRef::Node(node)) = node.word_separators.as_ref() {
        refs.extend(self.find_refs(node, event, true));
      }
    }

    refs
  }

  fn is_word_separator(&self, event: &Event) -> bool {
    match event {
      Event::Key { key, chars } => {
        if self.key_word_separators.contains(key) {
          true
        } else if let Some(char) = chars {
          self.char_word_separators.contains(char)
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
  use crate::rolling::StringMatchOptions;
  use crate::util::tests::get_matches_after_str;

  fn match_result<Id: Default>(id: Id, trigger: &str) -> MatchResult<Id> {
    MatchResult {
      id,
      trigger: trigger.to_string(),
      ..Default::default()
    }
  }

  fn match_result_with_sep<Id: Default>(
    id: Id,
    trigger: &str,
    left: Option<&str>,
    right: Option<&str>,
  ) -> MatchResult<Id> {
    MatchResult {
      id,
      trigger: trigger.to_string(),
      left_separator: left.map(str::to_owned),
      right_separator: right.map(str::to_owned),
      ..Default::default()
    }
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

    assert_eq!(
      get_matches_after_str("hi", &matcher),
      vec![match_result(1, "hi"), match_result(5, "hi")]
    );
    assert_eq!(
      get_matches_after_str("my", &matcher),
      vec![match_result(3, "my")]
    );
    assert_eq!(
      get_matches_after_str("mmy", &matcher),
      vec![match_result(3, "my")]
    );
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
    // Word matches are also triggered when there is no left separator but it's a new state
    assert_eq!(
      get_matches_after_str("hi,", &matcher),
      vec![match_result_with_sep(1, "hi,", None, Some(","))]
    );
    assert_eq!(
      get_matches_after_str(".hi,", &matcher),
      vec![match_result_with_sep(1, ".hi,", Some("."), Some(","))]
    );
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
            ..Default::default()
          },
        ),
      ],
      RollingMatcherOptions {
        char_word_separators: vec![".".to_string(), ",".to_string()],
        ..Default::default()
      },
    );

    assert_eq!(
      get_matches_after_str("hi", &matcher),
      vec![match_result(1, "hi")]
    );
    assert_eq!(
      get_matches_after_str("Hi", &matcher),
      vec![match_result(1, "Hi")]
    );
    assert_eq!(
      get_matches_after_str("HI", &matcher),
      vec![match_result(1, "HI")]
    );
    assert_eq!(
      get_matches_after_str("arty", &matcher),
      vec![match_result(3, "arty")]
    );
    assert_eq!(
      get_matches_after_str("arTY", &matcher),
      vec![match_result(3, "arTY")]
    );
    assert_eq!(
      get_matches_after_str("ARTY", &matcher),
      vec![match_result(3, "ARTY")]
    );
  }
}

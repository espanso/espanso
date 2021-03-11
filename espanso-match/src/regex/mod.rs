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

use log::error;
use regex::{Regex, RegexSet};

use crate::event::Event;
use crate::Matcher;

#[derive(Debug)]
pub struct RegexMatch<Id> {
  pub id: Id,
  pub regex: String,
}

impl<Id> RegexMatch<Id> {
  pub fn new(id: Id, regex: &str) -> Self {
    Self {
      id,
      regex: regex.to_string(),
    }
  }
}

#[derive(Clone)]
pub struct RegexMatcherState {
  buffer: String,
}

impl Default for RegexMatcherState {
  fn default() -> Self {
    Self {
      buffer: String::new(),
    }
  }
}

pub struct RegexMatcher<Id> {
  ids: Vec<Id>,
  // The RegexSet is used to efficiently determine which regexes match
  regex_set: RegexSet,

  // The single regexes are then used to find the captures
  regexes: Vec<Regex>,
}

impl<'a, Id> Matcher<'a, RegexMatcherState, Id> for RegexMatcher<Id>
where
  Id: Clone,
{
  fn process(
    &'a self,
    prev_state: Option<&RegexMatcherState>,
    event: Event,
  ) -> (RegexMatcherState, Vec<Id>) {
    let mut buffer = if let Some(prev_state) = prev_state {
      prev_state.buffer.clone()
    } else {
      "".to_string()
    };

    if let Event::Key { key: _, chars } = event {
      if let Some(chars) = chars {
        buffer.push_str(&chars);
      }
    }

    // Find matches
    if self.regex_set.is_match(&buffer) {
      for index in self.regex_set.matches(&buffer) {
        if let (Some(id), Some(regex)) = (self.ids.get(index), self.regexes.get(index)) {
          // TODO: find complete match and captures
        } else {
          error!("received inconsistent index from regex set with index: {}", index);
        }
      }
    }

    let current_state = RegexMatcherState { buffer };
    (current_state, Vec::new())
  }
}

impl<Id: Clone> RegexMatcher<Id> {
  pub fn new(matches: &[RegexMatch<Id>]) -> Self {
    let mut ids = Vec::new();
    let mut regexes = Vec::new();
    let mut good_regexes = Vec::new();

    for m in matches {
      match Regex::new(&m.regex) {
        Ok(regex) => {
          ids.push(m.id.clone());
          good_regexes.push(&m.regex);
          regexes.push(regex);
        }
        Err(err) => {
          error!("unable to compile regex: '{}', error: {:?}", m.regex, err);
        }
      }
    }

    let regex_set = RegexSet::new(&good_regexes).expect("unable to build regex set");
    
    Self { ids, regex_set, regexes }
  }
}

// TODO: test
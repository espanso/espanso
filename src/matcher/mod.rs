/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

use serde::{Serialize, Deserialize, Deserializer};
use crate::event::{KeyEvent, KeyModifier};
use crate::event::KeyEventReceiver;
use serde_yaml::Mapping;
use regex::Regex;

pub(crate) mod scrolling;

#[derive(Debug, Serialize, Clone)]
pub struct Match {
    pub trigger: String,
    pub replace: String,
    pub vars: Vec<MatchVariable>,
    pub word: bool,

    #[serde(skip_serializing)]
    pub _has_vars: bool,

    // Automatically calculated from the trigger, used by the matcher to check for correspondences.
    #[serde(skip_serializing)]
    pub _trigger_sequence: Vec<TriggerEntry>,
}

impl <'de> serde::Deserialize<'de> for Match {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de> {

        let auto_match = AutoMatch::deserialize(deserializer)?;
        Ok(Match::from(&auto_match))
    }
}

impl<'a> From<&'a AutoMatch> for Match{
    fn from(other: &'a AutoMatch) -> Self {
        lazy_static! {
            static ref VAR_REGEX: Regex = Regex::new("\\{\\{\\s*(\\w+)\\s*\\}\\}").unwrap();
        }

        // TODO: may need to replace windows newline (\r\n) with newline only (\n)

        let new_replace = other.replace.clone();

        // Check if the match contains variables
        let has_vars = VAR_REGEX.is_match(&other.replace);

        // Calculate the trigger sequence
        let mut trigger_sequence = Vec::new();
        let trigger_chars : Vec<char> = other.trigger.chars().collect();
        trigger_sequence.extend(trigger_chars.into_iter().map(|c| {
            TriggerEntry::Char(c)
        }));
        if other.word {  // If it's a word match, end with a word separator
            trigger_sequence.push(TriggerEntry::WordSeparator);
        }

        Self {
            trigger: other.trigger.clone(),
            replace: new_replace,
            vars: other.vars.clone(),
            word: other.word.clone(),
            _has_vars: has_vars,
            _trigger_sequence: trigger_sequence,
        }
    }
}

/// Used to deserialize the Match struct before applying some custom elaboration.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AutoMatch {
    pub trigger: String,
    pub replace: String,

    #[serde(default = "default_vars")]
    pub vars: Vec<MatchVariable>,

    #[serde(default = "default_word")]
    pub word: bool,
}

fn default_vars() -> Vec<MatchVariable> {Vec::new()}
fn default_word() -> bool {false}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchVariable {
    pub name: String,

    #[serde(rename = "type")]
    pub var_type: String,

    pub params: Mapping,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TriggerEntry {
    Char(char),
    WordSeparator
}

pub trait MatchReceiver {
    fn on_match(&self, m: &Match, trailing_separator: Option<char>);
    fn on_enable_update(&self, status: bool);
}

pub trait Matcher : KeyEventReceiver {
    fn handle_char(&self, c: &str);
    fn handle_modifier(&self, m: KeyModifier);
}

impl <M: Matcher> KeyEventReceiver for M {
    fn on_key_event(&self, e: KeyEvent) {
        match e {
            KeyEvent::Char(c) => {
                self.handle_char(&c);
            },
            KeyEvent::Modifier(m) => {
                self.handle_modifier(m);
            },
        }
    }
}


// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_has_vars_should_be_false() {
        let match_str = r###"
        trigger: ":test"
        replace: "There are no variables"
        "###;

        let _match : Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match._has_vars, false);
    }

    #[test]
    fn test_match_has_vars_should_be_true() {
        let match_str = r###"
        trigger: ":test"
        replace: "There are {{one}} and {{two}} variables"
        "###;

        let _match : Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match._has_vars, true);
    }

    #[test]
    fn test_match_has_vars_with_spaces_should_be_true() {
        let match_str = r###"
        trigger: ":test"
        replace: "There is {{ one }} variable"
        "###;

        let _match : Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match._has_vars, true);
    }

    #[test]
    fn test_match_trigger_sequence_without_word() {
        let match_str = r###"
        trigger: "test"
        replace: "This is a test"
        "###;

        let _match : Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match._trigger_sequence[0], TriggerEntry::Char('t'));
        assert_eq!(_match._trigger_sequence[1], TriggerEntry::Char('e'));
        assert_eq!(_match._trigger_sequence[2], TriggerEntry::Char('s'));
        assert_eq!(_match._trigger_sequence[3], TriggerEntry::Char('t'));
    }

    #[test]
    fn test_match_trigger_sequence_with_word() {
        let match_str = r###"
        trigger: "test"
        replace: "This is a test"
        word: true
        "###;

        let _match : Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match._trigger_sequence[0], TriggerEntry::Char('t'));
        assert_eq!(_match._trigger_sequence[1], TriggerEntry::Char('e'));
        assert_eq!(_match._trigger_sequence[2], TriggerEntry::Char('s'));
        assert_eq!(_match._trigger_sequence[3], TriggerEntry::Char('t'));
        assert_eq!(_match._trigger_sequence[4], TriggerEntry::WordSeparator);
    }
}
/*
 * This file is part of espans{ name: (), var_type: (), params: ()}
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

use crate::event::KeyEventReceiver;
use crate::event::{KeyEvent, KeyModifier};
use regex::{Captures, Regex};
use serde::{Deserialize, Deserializer, Serialize};
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::PathBuf;
use std::{borrow::Cow, collections::HashMap};

pub(crate) mod scrolling;

#[derive(Debug, Serialize, Clone)]
pub struct Match {
    pub triggers: Vec<String>,
    pub content: MatchContentType,
    pub word: bool,
    pub passive_only: bool,
    pub propagate_case: bool,
    pub force_clipboard: bool,
    pub is_html: bool,

    // Automatically calculated from the triggers, used by the matcher to check for correspondences.
    #[serde(skip_serializing)]
    pub _trigger_sequences: Vec<Vec<TriggerEntry>>,
}

#[derive(Debug, Serialize, Clone)]
pub enum MatchContentType {
    Text(TextContent),
    Image(ImageContent),
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct TextContent {
    pub replace: String,
    pub vars: Vec<MatchVariable>,

    #[serde(skip_serializing)]
    pub _has_vars: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ImageContent {
    pub path: PathBuf,
}

impl<'de> serde::Deserialize<'de> for Match {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let auto_match = AutoMatch::deserialize(deserializer)?;
        Ok(Match::from(&auto_match))
    }
}

impl<'a> From<&'a AutoMatch> for Match {
    fn from(other: &'a AutoMatch) -> Self {
        lazy_static! {
            static ref VAR_REGEX: Regex =
                Regex::new("\\{\\{\\s*(\\w+)(\\.\\w+)?\\s*\\}\\}").unwrap();
        };

        let mut triggers = if !other.triggers.is_empty() {
            other.triggers.clone()
        } else if !other.trigger.is_empty() {
            vec![other.trigger.clone()]
        } else {
            panic!("Match does not have any trigger defined: {:?}", other)
        };

        // If propagate_case is true, we need to generate all the possible triggers
        // For example, specifying "hello" as a trigger, we need to have:
        // "hello", "Hello", "HELLO"
        if other.propagate_case {
            // List with first letter capitalized
            let first_capitalized: Vec<String> = triggers
                .iter()
                .map(|trigger| {
                    let capitalized = trigger.clone();
                    let mut v: Vec<char> = capitalized.chars().collect();

                    // Capitalize the first alphabetic letter
                    // See issue #244
                    let first_alphabetic = v.iter().position(|c| c.is_alphabetic()).unwrap_or(0);

                    v[first_alphabetic] = v[first_alphabetic].to_uppercase().nth(0).unwrap();
                    v.into_iter().collect()
                })
                .collect();

            let all_capitalized: Vec<String> = triggers
                .iter()
                .map(|trigger| trigger.to_uppercase())
                .collect();

            triggers.extend(first_capitalized);
            triggers.extend(all_capitalized);
        }

        let trigger_sequences = triggers
            .iter()
            .map(|trigger| {
                // Calculate the trigger sequence
                let mut trigger_sequence = Vec::new();
                let trigger_chars: Vec<char> = trigger.chars().collect();
                trigger_sequence.extend(trigger_chars.into_iter().map(|c| TriggerEntry::Char(c)));
                if other.word {
                    // If it's a word match, end with a word separator
                    trigger_sequence.push(TriggerEntry::WordSeparator);
                }

                trigger_sequence
            })
            .collect();

        let (text_content, is_html) = if let Some(replace) = &other.replace {
            (Some(Cow::from(replace)), false)
        } else if let Some(markdown_str) = &other.markdown {
            // Render the markdown into HTML
            let mut html = markdown::to_html(markdown_str);
            html = html.trim().to_owned();

            if !other.paragraph {
                // Remove the surrounding paragraph
                if html.starts_with("<p>") {
                    html = html.trim_start_matches("<p>").to_owned();
                }
                if html.ends_with("</p>") {
                    html = html.trim_end_matches("</p>").to_owned();
                }
            }

            (Some(Cow::from(html)), true)
        } else if let Some(html) = &other.html {
            (Some(Cow::from(html)), true)
        } else {
            (None, false)
        };

        let content = if let Some(content) = text_content {
            // Check if the match contains variables
            let has_vars = VAR_REGEX.is_match(&content);

            let content = TextContent {
                replace: content.to_string(),
                vars: other.vars.clone(),
                _has_vars: has_vars,
            };

            MatchContentType::Text(content)
        } else if let Some(form) = &other.form {
            // Form shorthand
            // Replace all the form fields with actual variables
            let new_replace = VAR_REGEX.replace_all(&form, |caps: &Captures| {
                let var_name = caps.get(1).unwrap().as_str();
                format!("{{{{form1.{}}}}}", var_name)
            });
            let new_replace = new_replace.to_string();

            // Convert the form data to valid variables
            let mut params = Mapping::new();
            if let Some(fields) = &other.form_fields {
                let mut mapping_fields = Mapping::new();
                fields.iter().for_each(|(key, value)| {
                    mapping_fields.insert(Value::from(key.to_owned()), Value::from(value.clone()));
                });
                params.insert(Value::from("fields"), Value::from(mapping_fields));
            }
            params.insert(Value::from("layout"), Value::from(form.to_owned()));

            let vars = vec![MatchVariable {
                name: "form1".to_owned(),
                var_type: "form".to_owned(),
                params,
            }];

            let content = TextContent {
                replace: new_replace,
                vars,
                _has_vars: true,
            };

            MatchContentType::Text(content)
        } else if let Some(image_path) = &other.image_path {
            // Image match
            // On Windows, we have to replace the forward / with the backslash \ in the path
            let new_path = if cfg!(target_os = "windows") {
                image_path.replace("/", "\\")
            } else {
                image_path.to_owned()
            };

            // Calculate variables in path
            let new_path = if new_path.contains("$CONFIG") {
                let config_dir = crate::context::get_config_dir();
                let config_path = fs::canonicalize(&config_dir);
                let config_path = if let Ok(config_path) = config_path {
                    config_path.to_string_lossy().into_owned()
                } else {
                    "".to_owned()
                };
                new_path.replace("$CONFIG", &config_path)
            } else {
                new_path.to_owned()
            };

            let content = ImageContent {
                path: PathBuf::from(new_path),
            };

            MatchContentType::Image(content)
        } else {
            eprintln!("ERROR: no action specified for match {}, please specify either 'replace', 'markdown', 'html', image_path' or 'form'", other.trigger);
            std::process::exit(2);
        };

        Self {
            triggers,
            content,
            word: other.word,
            passive_only: other.passive_only,
            _trigger_sequences: trigger_sequences,
            propagate_case: other.propagate_case,
            force_clipboard: other.force_clipboard,
            is_html,
        }
    }
}

/// Used to deserialize the Match struct before applying some custom elaboration.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AutoMatch {
    #[serde(default = "default_trigger")]
    pub trigger: String,

    #[serde(default = "default_triggers")]
    pub triggers: Vec<String>,

    #[serde(default = "default_replace")]
    pub replace: Option<String>,

    #[serde(default = "default_image_path")]
    pub image_path: Option<String>,

    #[serde(default = "default_form")]
    pub form: Option<String>,

    #[serde(default = "default_form_fields")]
    pub form_fields: Option<HashMap<String, Value>>,

    #[serde(default = "default_vars")]
    pub vars: Vec<MatchVariable>,

    #[serde(default = "default_word")]
    pub word: bool,

    #[serde(default = "default_passive_only")]
    pub passive_only: bool,

    #[serde(default = "default_propagate_case")]
    pub propagate_case: bool,

    #[serde(default = "default_force_clipboard")]
    pub force_clipboard: bool,

    #[serde(default = "default_markdown")]
    pub markdown: Option<String>,

    #[serde(default = "default_paragraph")]
    pub paragraph: bool,

    #[serde(default = "default_html")]
    pub html: Option<String>,
}

fn default_trigger() -> String {
    "".to_owned()
}
fn default_triggers() -> Vec<String> {
    Vec::new()
}
fn default_vars() -> Vec<MatchVariable> {
    Vec::new()
}
fn default_word() -> bool {
    false
}
fn default_passive_only() -> bool {
    false
}
fn default_replace() -> Option<String> {
    None
}
fn default_form() -> Option<String> {
    None
}
fn default_form_fields() -> Option<HashMap<String, Value>> {
    None
}
fn default_image_path() -> Option<String> {
    None
}
fn default_propagate_case() -> bool {
    false
}
fn default_force_clipboard() -> bool {
    false
}
fn default_markdown() -> Option<String> {
    None
}
fn default_paragraph() -> bool {
    true
}
fn default_html() -> Option<String> {
    None
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MatchVariable {
    pub name: String,

    #[serde(rename = "type")]
    pub var_type: String,

    #[serde(default = "default_params")]
    pub params: Mapping,
}

fn default_params() -> Mapping {
    Mapping::new()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TriggerEntry {
    Char(char),
    WordSeparator,
}

pub trait MatchReceiver {
    fn on_match(&self, m: &Match, trailing_separator: Option<char>, trigger_offset: usize);
    fn on_enable_update(&self, status: bool);
    fn on_passive(&self);
    fn on_undo(&self);
}

pub trait Matcher: KeyEventReceiver {
    fn handle_char(&self, c: &str);
    fn handle_modifier(&self, m: KeyModifier);
    fn handle_other(&self);
}

impl<M: Matcher> KeyEventReceiver for M {
    fn on_key_event(&self, e: KeyEvent) {
        match e {
            KeyEvent::Char(c) => {
                self.handle_char(&c);
            }
            KeyEvent::Modifier(m) => {
                self.handle_modifier(m);
            }
            KeyEvent::Other => {
                self.handle_other();
            }
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

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        match _match.content {
            MatchContentType::Text(content) => {
                assert_eq!(content._has_vars, false);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_match_has_vars_should_be_true() {
        let match_str = r###"
        trigger: ":test"
        replace: "There are {{one}} and {{two}} variables"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        match _match.content {
            MatchContentType::Text(content) => {
                assert_eq!(content._has_vars, true);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_match_has_vars_with_spaces_should_be_true() {
        let match_str = r###"
        trigger: ":test"
        replace: "There is {{ one }} variable"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        match _match.content {
            MatchContentType::Text(content) => {
                assert_eq!(content._has_vars, true);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_match_trigger_sequence_without_word() {
        let match_str = r###"
        trigger: "test"
        replace: "This is a test"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match._trigger_sequences[0][0], TriggerEntry::Char('t'));
        assert_eq!(_match._trigger_sequences[0][1], TriggerEntry::Char('e'));
        assert_eq!(_match._trigger_sequences[0][2], TriggerEntry::Char('s'));
        assert_eq!(_match._trigger_sequences[0][3], TriggerEntry::Char('t'));
    }

    #[test]
    fn test_match_trigger_sequence_with_word() {
        let match_str = r###"
        trigger: "test"
        replace: "This is a test"
        word: true
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match._trigger_sequences[0][0], TriggerEntry::Char('t'));
        assert_eq!(_match._trigger_sequences[0][1], TriggerEntry::Char('e'));
        assert_eq!(_match._trigger_sequences[0][2], TriggerEntry::Char('s'));
        assert_eq!(_match._trigger_sequences[0][3], TriggerEntry::Char('t'));
        assert_eq!(_match._trigger_sequences[0][4], TriggerEntry::WordSeparator);
    }

    #[test]
    fn test_match_with_image_content() {
        let match_str = r###"
        trigger: "test"
        image_path: "/path/to/file"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        match _match.content {
            MatchContentType::Image(content) => {
                assert_eq!(content.path, PathBuf::from("/path/to/file"));
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_match_trigger_populates_triggers_vector() {
        let match_str = r###"
        trigger: ":test"
        replace: "This is a test"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match.triggers, vec![":test"])
    }

    #[test]
    fn test_match_triggers_are_correctly_parsed() {
        let match_str = r###"
        triggers:
          - ":test1"
          - :test2
        replace: "This is a test"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match.triggers, vec![":test1", ":test2"])
    }

    #[test]
    fn test_match_triggers_are_correctly_parsed_square_brackets() {
        let match_str = r###"
        triggers: [":test1", ":test2"]
        replace: "This is a test"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match.triggers, vec![":test1", ":test2"])
    }

    #[test]
    fn test_match_propagate_case() {
        let match_str = r###"
        trigger: "hello"
        replace: "This is a test"
        propagate_case: true
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match.triggers, vec!["hello", "Hello", "HELLO"])
    }

    #[test]
    fn test_match_propagate_case_multi_trigger() {
        let match_str = r###"
        triggers: ["hello", "hi"]
        replace: "This is a test"
        propagate_case: true
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(
            _match.triggers,
            vec!["hello", "hi", "Hello", "Hi", "HELLO", "HI"]
        )
    }

    #[test]
    fn test_match_trigger_sequence_with_word_propagate_case() {
        let match_str = r###"
        trigger: "test"
        replace: "This is a test"
        word: true
        propagate_case: true
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match._trigger_sequences[0][0], TriggerEntry::Char('t'));
        assert_eq!(_match._trigger_sequences[0][1], TriggerEntry::Char('e'));
        assert_eq!(_match._trigger_sequences[0][2], TriggerEntry::Char('s'));
        assert_eq!(_match._trigger_sequences[0][3], TriggerEntry::Char('t'));
        assert_eq!(_match._trigger_sequences[0][4], TriggerEntry::WordSeparator);

        assert_eq!(_match._trigger_sequences[1][0], TriggerEntry::Char('T'));
        assert_eq!(_match._trigger_sequences[1][1], TriggerEntry::Char('e'));
        assert_eq!(_match._trigger_sequences[1][2], TriggerEntry::Char('s'));
        assert_eq!(_match._trigger_sequences[1][3], TriggerEntry::Char('t'));
        assert_eq!(_match._trigger_sequences[1][4], TriggerEntry::WordSeparator);

        assert_eq!(_match._trigger_sequences[2][0], TriggerEntry::Char('T'));
        assert_eq!(_match._trigger_sequences[2][1], TriggerEntry::Char('E'));
        assert_eq!(_match._trigger_sequences[2][2], TriggerEntry::Char('S'));
        assert_eq!(_match._trigger_sequences[2][3], TriggerEntry::Char('T'));
        assert_eq!(_match._trigger_sequences[2][4], TriggerEntry::WordSeparator);
    }

    #[test]
    fn test_match_empty_replace_doesnt_crash() {
        let match_str = r###"
        trigger: "hello"
        replace: ""
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();
    }

    #[test]
    fn test_match_propagate_case_with_prefix_symbol() {
        let match_str = r###"
        trigger: ":hello"
        replace: "This is a test"
        propagate_case: true
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match.triggers, vec![":hello", ":Hello", ":HELLO"])
    }

    #[test]
    fn test_match_propagate_case_non_alphabetic_should_not_crash() {
        let match_str = r###"
        trigger: ":.."
        replace: "This is a test"
        propagate_case: true
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        assert_eq!(_match.triggers, vec![":..", ":..", ":.."])
    }

    #[test]
    fn test_match_form_translated_correctly() {
        let match_str = r###"
        trigger: ":test"
        form: "Hey {{name}}, how are you? {{greet}}"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();
        match _match.content {
            MatchContentType::Text(content) => {
                let mut mapping = Mapping::new();
                mapping.insert(
                    Value::from("layout"),
                    Value::from("Hey {{name}}, how are you? {{greet}}"),
                );
                assert_eq!(
                    content,
                    TextContent {
                        replace: "Hey {{form1.name}}, how are you? {{form1.greet}}".to_owned(),
                        _has_vars: true,
                        vars: vec![MatchVariable {
                            name: "form1".to_owned(),
                            var_type: "form".to_owned(),
                            params: mapping,
                        }]
                    }
                );
            }
            _ => panic!("wrong content"),
        }
    }

    #[test]
    fn test_match_form_with_fields_translated_correctly() {
        let match_str = r###"
        trigger: ":test"
        form: "Hey {{name}}, how are you? {{greet}}"
        form_fields:
          name:
            multiline: true
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();
        match _match.content {
            MatchContentType::Text(content) => {
                let mut name_mapping = Mapping::new();
                name_mapping.insert(Value::from("multiline"), Value::Bool(true));
                let mut submapping = Mapping::new();
                submapping.insert(Value::from("name"), Value::from(name_mapping));
                let mut mapping = Mapping::new();
                mapping.insert(Value::from("fields"), Value::from(submapping));
                mapping.insert(
                    Value::from("layout"),
                    Value::from("Hey {{name}}, how are you? {{greet}}"),
                );
                assert_eq!(
                    content,
                    TextContent {
                        replace: "Hey {{form1.name}}, how are you? {{form1.greet}}".to_owned(),
                        _has_vars: true,
                        vars: vec![MatchVariable {
                            name: "form1".to_owned(),
                            var_type: "form".to_owned(),
                            params: mapping,
                        }]
                    }
                );
            }
            _ => panic!("wrong content"),
        }
    }

    #[test]
    fn test_match_markdown_loaded_correctly() {
        let match_str = r###"
        trigger: ":test"
        markdown: "This *text* is **very bold**"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        match _match.content {
            MatchContentType::Text(content) => {
                assert_eq!(
                    content.replace,
                    "This <em>text</em> is <strong>very bold</strong>"
                );
                assert_eq!(_match.is_html, true);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_match_markdown_keep_vars() {
        let match_str = r###"
        trigger: ":test"
        markdown: "This *text* is {{variable}} **very bold**"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        match _match.content {
            MatchContentType::Text(content) => {
                assert_eq!(
                    content.replace,
                    "This <em>text</em> is {{variable}} <strong>very bold</strong>"
                );
                assert_eq!(_match.is_html, true);
                assert_eq!(content._has_vars, true);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_match_html_loaded_correctly() {
        let match_str = r###"
        trigger: ":test"
        html: "This <i>text<i> is <b>very bold</b>"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        match _match.content {
            MatchContentType::Text(content) => {
                assert_eq!(content.replace, "This <i>text<i> is <b>very bold</b>");
                assert_eq!(_match.is_html, true);
            }
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_match_html_keep_vars() {
        let match_str = r###"
        trigger: ":test"
        html: "This <i>text<i> is {{var}} <b>very bold</b>"
        "###;

        let _match: Match = serde_yaml::from_str(match_str).unwrap();

        match _match.content {
            MatchContentType::Text(content) => {
                assert_eq!(
                    content.replace,
                    "This <i>text<i> is {{var}} <b>very bold</b>"
                );
                assert_eq!(_match.is_html, true);
                assert_eq!(content._has_vars, true);
            }
            _ => {
                assert!(false);
            }
        }
    }
}

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

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_norway::Mapping;

use crate::util::is_yaml_empty;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YAMLMatchGroup {
    #[serde(default)]
    pub imports: Option<Vec<String>>,

    #[serde(default)]
    pub global_vars: Option<Vec<YAMLVariable>>,

    #[serde(default)]
    pub match_defaults: Option<YAMLMatchDefaults>,

    #[serde(default)]
    pub matches: Option<Vec<YAMLMatch>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YAMLMatchDefaults {
    #[serde(default)]
    pub word: Option<bool>,

    #[serde(default)]
    pub left_word: Option<bool>,

    #[serde(default)]
    pub right_word: Option<bool>,

    #[serde(default)]
    pub propagate_case: Option<bool>,

    #[serde(default)]
    pub uppercase_style: Option<String>,

    #[serde(default)]
    pub force_clipboard: Option<bool>,

    #[serde(default)]
    pub force_mode: Option<String>,

    #[serde(default)]
    pub triggermarker_prefix: Option<String>,

    #[serde(default)]
    pub triggermarker_suffix: Option<String>,

    #[serde(default)]
    pub triggermarker_replace_mode: Option<String>,

    #[serde(default)]
    pub triggermarker_prefix_replace_mode: Option<String>,

    #[serde(default)]
    pub triggermarker_suffix_replace_mode: Option<String>,

    #[serde(default)]
    pub triggermarker_smart_chars: Option<Vec<String>>,

    #[serde(default)]
    pub triggermarker_smart_remove_multiple: Option<bool>,
}

impl YAMLMatchGroup {
    pub fn parse_from_str(yaml: &str) -> Result<Self> {
        // Remove UTF-8 BOM if present (common in Windows editors like Notepad)
        let yaml = yaml.trim_start_matches('\u{FEFF}');

        // Because an empty string is not valid YAML but we want to support it anyway
        if is_yaml_empty(yaml) {
            return Ok(serde_norway::from_str(
                "arbitrary_field_that_will_not_block_the_parser: true",
            )?);
        }

        Ok(serde_norway::from_str(yaml)?)
    }

    pub fn parse_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_from_str(&content)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YAMLMatch {
    #[serde(default)]
    pub label: Option<String>,

    #[serde(default)]
    pub trigger: Option<String>,

    #[serde(default)]
    pub triggers: Option<Vec<String>>,

    #[serde(default)]
    pub regex: Option<String>,

    #[serde(default)]
    pub replace: Option<String>,

    #[serde(default)]
    pub image_path: Option<String>,

    #[serde(default)]
    pub form: Option<String>,

    #[serde(default)]
    pub form_fields: Option<Mapping>,

    #[serde(default)]
    pub vars: Option<Vec<YAMLVariable>>,

    #[serde(default)]
    pub word: Option<bool>,

    #[serde(default)]
    pub left_word: Option<bool>,

    #[serde(default)]
    pub right_word: Option<bool>,

    #[serde(default)]
    pub propagate_case: Option<bool>,

    #[serde(default)]
    pub uppercase_style: Option<String>,

    #[serde(default)]
    pub force_clipboard: Option<bool>,

    #[serde(default)]
    pub force_mode: Option<String>,

    #[serde(default)]
    pub triggermarker_prefix: Option<String>,

    #[serde(default)]
    pub triggermarker_suffix: Option<String>,

    #[serde(default)]
    pub triggermarker_replace_mode: Option<String>,

    #[serde(default)]
    pub triggermarker_prefix_replace_mode: Option<String>,

    #[serde(default)]
    pub triggermarker_suffix_replace_mode: Option<String>,

    #[serde(default)]
    pub triggermarker_smart_chars: Option<Vec<String>>,

    #[serde(default)]
    pub triggermarker_smart_remove_multiple: Option<bool>,

    #[serde(default)]
    pub markdown: Option<String>,

    #[serde(default)]
    pub paragraph: Option<bool>,

    #[serde(default)]
    pub html: Option<String>,

    #[serde(default)]
    pub search_terms: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct YAMLVariable {
    pub name: String,

    #[serde(rename = "type")]
    pub var_type: String,

    #[serde(default = "default_params")]
    pub params: Mapping,

    #[serde(default)]
    pub inject_vars: Option<bool>,

    #[serde(default)]
    pub depends_on: Vec<String>,
}

fn default_params() -> Mapping {
    Mapping::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_match_defaults_basic() {
        let yaml = r#"
match_defaults:
  word: true
  propagate_case: true
  force_clipboard: false
  uppercase_style: "capitalize"

matches:
  - trigger: ":hello"
    replace: "Hello World!"
"#;

        let result = YAMLMatchGroup::parse_from_str(yaml);
        assert!(result.is_ok(), "Failed to parse YAML: {:?}", result.err());

        let group = result.unwrap();
        assert!(group.match_defaults.is_some());

        let defaults = group.match_defaults.unwrap();
        assert_eq!(defaults.word, Some(true));
        assert_eq!(defaults.propagate_case, Some(true));
        assert_eq!(defaults.force_clipboard, Some(false));
        assert_eq!(defaults.uppercase_style, Some("capitalize".to_string()));
    }

    #[test]
    fn test_parse_match_defaults_all_fields() {
        let yaml = r#"
match_defaults:
  word: false
  left_word: true
  right_word: false
  propagate_case: true
  uppercase_style: "uppercase"
  force_clipboard: true
  force_mode: "keys"
"#;

        let result = YAMLMatchGroup::parse_from_str(yaml);
        assert!(result.is_ok(), "Failed to parse YAML: {:?}", result.err());

        let group = result.unwrap();
        assert!(group.match_defaults.is_some());

        let defaults = group.match_defaults.unwrap();
        assert_eq!(defaults.word, Some(false));
        assert_eq!(defaults.left_word, Some(true));
        assert_eq!(defaults.right_word, Some(false));
        assert_eq!(defaults.propagate_case, Some(true));
        assert_eq!(defaults.uppercase_style, Some("uppercase".to_string()));
        assert_eq!(defaults.force_clipboard, Some(true));
        assert_eq!(defaults.force_mode, Some("keys".to_string()));
    }

    #[test]
    fn test_parse_empty_match_defaults() {
        let yaml = r#"
match_defaults: {}

matches:
  - trigger: ":test"
    replace: "value"
"#;

        let result = YAMLMatchGroup::parse_from_str(yaml);
        assert!(result.is_ok(), "Failed to parse YAML: {:?}", result.err());

        let group = result.unwrap();
        assert!(group.match_defaults.is_some());

        let defaults = group.match_defaults.unwrap();
        assert_eq!(defaults.word, None);
        assert_eq!(defaults.left_word, None);
        assert_eq!(defaults.right_word, None);
        assert_eq!(defaults.propagate_case, None);
        assert_eq!(defaults.uppercase_style, None);
        assert_eq!(defaults.force_clipboard, None);
        assert_eq!(defaults.force_mode, None);
    }

    #[test]
    fn test_parse_no_match_defaults() {
        let yaml = r#"
matches:
  - trigger: ":test"
    replace: "value"
"#;

        let result = YAMLMatchGroup::parse_from_str(yaml);
        assert!(result.is_ok(), "Failed to parse YAML: {:?}", result.err());

        let group = result.unwrap();
        assert!(group.match_defaults.is_none());
    }

    #[test]
    fn test_parse_with_utf8_bom() {
        let yaml = "\u{FEFF}match_defaults:\n  word: true\n";

        let result = YAMLMatchGroup::parse_from_str(yaml);
        assert!(
            result.is_ok(),
            "Failed to parse YAML with BOM: {:?}",
            result.err()
        );

        let group = result.unwrap();
        assert!(group.match_defaults.is_some());
        assert_eq!(group.match_defaults.unwrap().word, Some(true));
    }

    #[test]
    fn test_parse_triggermarker_defaults_basic() {
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
match_defaults:
  triggermarker_prefix: ":"
  triggermarker_suffix: ";"
matches:
  - trigger: "test"
    replace: "replacement"
"#,
        )
        .unwrap();

        let defaults = yaml_group.match_defaults.as_ref().unwrap();
        assert_eq!(defaults.triggermarker_prefix, Some(":".to_string()));
        assert_eq!(defaults.triggermarker_suffix, Some(";".to_string()));
    }

    #[test]
    fn test_parse_triggermarker_modes() {
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
match_defaults:
  triggermarker_replace_mode: "smart"
  triggermarker_prefix_replace_mode: "agnostic"
  triggermarker_smart_chars: [":", ";"]
  triggermarker_smart_remove_multiple: true
matches:
  - trigger: "test"
    replace: "replacement"
"#,
        )
        .unwrap();

        let defaults = yaml_group.match_defaults.as_ref().unwrap();
        assert_eq!(defaults.triggermarker_replace_mode, Some("smart".to_string()));
        assert_eq!(defaults.triggermarker_prefix_replace_mode, Some("agnostic".to_string()));
        assert_eq!(defaults.triggermarker_smart_chars, Some(vec![":".to_string(), ";".to_string()]));
        assert_eq!(defaults.triggermarker_smart_remove_multiple, Some(true));
    }

    #[test]
    fn test_parse_triggermarker_in_match() {
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
matches:
  - trigger: "test"
    replace: "replacement"
    triggermarker_prefix: "!"
    triggermarker_suffix: "."
"#,
        )
        .unwrap();

        let yaml_match = &yaml_group.matches.as_ref().unwrap()[0];
        assert_eq!(yaml_match.triggermarker_prefix, Some("!".to_string()));
        assert_eq!(yaml_match.triggermarker_suffix, Some(".".to_string()));
    }
}

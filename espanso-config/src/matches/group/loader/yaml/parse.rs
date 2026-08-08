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

use crate::util::{is_yaml_empty, parse_lenient};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YAMLMatchGroup {
    #[serde(default)]
    pub imports: Option<Vec<String>>,

    #[serde(default)]
    pub global_vars: Option<Vec<YAMLVariable>>,

    #[serde(default)]
    pub matches: Option<Vec<YAMLMatch>>,
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

        Ok(parse_lenient(yaml)?)
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

    // Regression for https://github.com/espanso/espanso/issues/2748
    // On v2.4.0 (serde_yaml -> serde_norway swap, PR #2532) an unquoted flow value
    // whose first character is a YAML flow indicator (`:`, `>`, ...) is rejected by
    // libyaml: "did not find expected node content ... while parsing a flow node".
    #[test]
    fn parse_unquoted_flow_triggers_issue_2748() {
        let yaml = "matches:\n  - triggers: [:->,:>-]\n    replace: smile\n";
        let group = YAMLMatchGroup::parse_from_str(yaml).unwrap();
        let matches = group.matches.expect("expected matches");
        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches[0].triggers,
            Some(vec![":->".to_string(), ":>-".to_string()])
        );
        assert_eq!(matches[0].replace.as_deref(), Some("smile"));
    }

    #[test]
    fn parse_unquoted_flow_triggers_spaced_emoticons() {
        let yaml = "matches:\n  - triggers: [:-), :-D]\n    replace: smile\n";
        let group = YAMLMatchGroup::parse_from_str(yaml).unwrap();
        let triggers = group.matches.unwrap().remove(0).triggers.unwrap();
        assert_eq!(triggers, vec![":-)", ":-D"]);
    }

    #[test]
    fn parse_unquoted_flow_arrow_value() {
        let yaml = "matches:\n  - triggers: [:>hello]\n    replace: hi\n";
        let group = YAMLMatchGroup::parse_from_str(yaml).unwrap();
        let triggers = group.matches.unwrap().remove(0).triggers.unwrap();
        assert_eq!(triggers, vec![":>hello"]);
    }

    // Regression: a #2748 trigger combined with a `replace: |` block scalar whose
    // literal body contains flow-like text (`[ :>x ]`). The lenient transform must fix
    // the trigger WITHOUT touching the block-scalar body byte-for-byte.
    #[test]
    fn parse_unquoted_flow_trigger_with_block_scalar_body() {
        let yaml =
            "matches:\n  - triggers: [:-)]\n    replace: |\n      if [ :>x ]; then echo hi; fi\n";
        let group = YAMLMatchGroup::parse_from_str(yaml).unwrap();
        let m = group.matches.unwrap().remove(0);
        assert_eq!(m.triggers, Some(vec![":-)".to_string()]));
        assert_eq!(m.replace.as_deref(), Some("if [ :>x ]; then echo hi; fi\n"));
    }
}

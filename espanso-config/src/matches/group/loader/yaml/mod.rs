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

use std::sync::LazyLock;

use crate::{
    counter::next_id,
    error::{ErrorRecord, NonFatalErrorSet},
    matches::{
        group::{path::resolve_imports, MatchGroup},
        ImageEffect, Match, Params, RegexCause, TextFormat, TextInjectMode, UpperCasingStyle,
        Value, Variable,
    },
};
use anyhow::{anyhow, bail, Context, Result};
use parse::YAMLMatchGroup;
use regex::{Captures, Regex};

use self::{
    parse::{YAMLMatch, YAMLVariable},
    util::convert_params,
};
use crate::matches::{MatchCause, MatchEffect, TextEffect, TriggerCause};

use super::Importer;

pub mod parse;
mod util;

static VAR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("\\{\\{\\s*(\\w+)(\\.\\w+)?\\s*\\}\\}").unwrap());
static FORM_CONTROL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("\\[\\[\\s*(\\w+)(\\.\\w+)?\\s*\\]\\]").unwrap());

// Create an alias to make the meaning more explicit
type Warning = anyhow::Error;

pub struct YAMLImporter {}

impl YAMLImporter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Importer for YAMLImporter {
    fn is_supported(&self, extension: &str) -> bool {
        extension == "yaml" || extension == "yml"
    }

    fn load_group(
        &self,
        path: &std::path::Path,
        config: &dyn crate::config::Config,
    ) -> anyhow::Result<(crate::matches::group::MatchGroup, Option<NonFatalErrorSet>)> {
        let yaml_group =
            YAMLMatchGroup::parse_from_file(path).context("failed to parse YAML match group")?;

        let mut non_fatal_errors = Vec::new();

        let mut global_vars = Vec::new();
        for yaml_global_var in yaml_group.global_vars.clone().unwrap_or_default() {
            match try_convert_into_variable(yaml_global_var, false) {
                Ok((var, warnings)) => {
                    global_vars.push(var);
                    non_fatal_errors.extend(warnings.into_iter().map(ErrorRecord::warn));
                }
                Err(err) => {
                    non_fatal_errors.push(ErrorRecord::error(err));
                }
            }
        }

        let mut matches = Vec::new();
        for yaml_match in yaml_group.matches.clone().unwrap_or_default() {
            match try_convert_into_match(yaml_match, false, yaml_group.match_defaults.as_ref(), Some(config)) {
                Ok((m, warnings)) => {
                    matches.push(m);
                    non_fatal_errors.extend(warnings.into_iter().map(ErrorRecord::warn));
                }
                Err(err) => {
                    non_fatal_errors.push(ErrorRecord::error(err));
                }
            }
        }

        // Resolve imports
        let (resolved_imports, import_errors) =
            resolve_imports(path, &yaml_group.imports.unwrap_or_default())
                .context("failed to resolve YAML match group imports")?;
        non_fatal_errors.extend(import_errors);

        let non_fatal_error_set = if non_fatal_errors.is_empty() {
            None
        } else {
            Some(NonFatalErrorSet::new(path, non_fatal_errors))
        };

        Ok((
            MatchGroup {
                imports: resolved_imports,
                global_vars,
                matches,
            },
            non_fatal_error_set,
        ))
    }
}

// Helper function to apply both prefix and suffix to a trigger
fn apply_triggermarkers(
    trigger: &str,
    prefix: Option<&str>,
    suffix: Option<&str>,
    prefix_mode: &str,
    suffix_mode: &str,
    smart_chars: &[String],
    remove_multiple: bool,
) -> String {
    let mut result = trigger.to_string();

    // Apply prefix
    if let Some(prefix_str) = prefix {
        result = apply_prefix(
            &result,
            prefix_str,
            prefix_mode,
            smart_chars,
            remove_multiple,
        );
    }

    // Apply suffix
    if let Some(suffix_str) = suffix {
        result = apply_suffix(
            &result,
            suffix_str,
            suffix_mode,
            smart_chars,
            remove_multiple,
        );
    }

    result
}

// Helper function to apply prefix to a trigger
fn apply_prefix(
    trigger: &str,
    prefix: &str,
    mode: &str,
    smart_chars: &[String],
    remove_multiple: bool,
) -> String {
    if mode == "agnostic" {
        // Agnostic mode: Simply prepend
        format!("{}{}", prefix, trigger)
    } else {
        // Smart mode: Remove existing smart chars, then add prefix
        let mut cleaned = trigger.to_string();

        // Remove leading smart chars if prefix is set or if smart char exists
        if !prefix.is_empty() || cleaned.chars().next().map_or(false, |c| {
            smart_chars.iter().any(|s| s.chars().next() == Some(c))
        }) {
            cleaned = remove_leading_smart_chars(&cleaned, smart_chars, remove_multiple);
        }

        // Add new prefix
        format!("{}{}", prefix, cleaned)
    }
}

// Helper function to apply suffix to a trigger
fn apply_suffix(
    trigger: &str,
    suffix: &str,
    mode: &str,
    smart_chars: &[String],
    remove_multiple: bool,
) -> String {
    if mode == "agnostic" {
        // Agnostic mode: Simply append
        format!("{}{}", trigger, suffix)
    } else {
        // Smart mode: Remove existing smart chars, then add suffix
        let mut cleaned = trigger.to_string();

        // Remove trailing smart chars if suffix is set or if smart char exists
        if !suffix.is_empty() || cleaned.chars().next_back().map_or(false, |c| {
            smart_chars.iter().any(|s| s.chars().next() == Some(c))
        }) {
            cleaned = remove_trailing_smart_chars(&cleaned, smart_chars, remove_multiple);
        }

        // Add new suffix
        format!("{}{}", cleaned, suffix)
    }
}

// Helper function to remove leading smart characters
fn remove_leading_smart_chars(
    trigger: &str,
    smart_chars: &[String],
    remove_multiple: bool,
) -> String {
    let mut chars: Vec<char> = trigger.chars().collect();

    if chars.is_empty() {
        return trigger.to_string();
    }

    // Get first character
    let first_char = chars[0];

    // Check if it's a smart char
    let is_smart_char = smart_chars.iter().any(|s| {
        s.chars().next() == Some(first_char)
    });

    if !is_smart_char {
        return trigger.to_string();
    }

    // Remove characters
    if remove_multiple {
        // Remove all repeated identical leading characters
        let mut pos = 0;
        while pos < chars.len() && chars[pos] == first_char {
            pos += 1;
        }
        chars.drain(0..pos);
    } else {
        // Remove only first character
        chars.remove(0);
    }

    chars.into_iter().collect()
}

// Helper function to remove trailing smart characters
fn remove_trailing_smart_chars(
    trigger: &str,
    smart_chars: &[String],
    remove_multiple: bool,
) -> String {
    let mut chars: Vec<char> = trigger.chars().collect();

    if chars.is_empty() {
        return trigger.to_string();
    }

    // Get last character
    let last_char = chars[chars.len() - 1];

    // Check if it's a smart char
    let is_smart_char = smart_chars.iter().any(|s| {
        s.chars().next() == Some(last_char)
    });

    if !is_smart_char {
        return trigger.to_string();
    }

    // Remove characters
    if remove_multiple {
        // Remove all repeated identical trailing characters
        let mut pos = chars.len();
        while pos > 0 && chars[pos - 1] == last_char {
            pos -= 1;
        }
        chars.truncate(pos);
    } else {
        // Remove only last character
        chars.pop();
    }

    chars.into_iter().collect()
}

pub fn try_convert_into_match(
    yaml_match: YAMLMatch,
    use_compatibility_mode: bool, // TODO: unused variable. Remove from the codebase
    defaults: Option<&parse::YAMLMatchDefaults>,
    config: Option<&dyn crate::config::Config>,
) -> Result<(Match, Vec<Warning>)> {
    let mut warnings = Vec::new();

    // Check if uppercase_style is specified but propagate_case won't be true
    let has_uppercase_style = yaml_match.uppercase_style.is_some()
        || defaults.and_then(|d| d.uppercase_style.as_ref()).is_some();
    let final_propagate_case = yaml_match
        .propagate_case
        .or(defaults.and_then(|d| d.propagate_case))
        .unwrap_or(false);

    if has_uppercase_style && !final_propagate_case {
        warnings.push(anyhow!(
            "specifying the 'uppercase_style' option without 'propagate_case' has no effect"
        ));
    }

    let triggers = if let Some(trigger) = yaml_match.trigger {
        Some(vec![trigger])
    } else {
        yaml_match.triggers
    };

    // Filter out empty triggers and add a warning
    let triggers = if let Some(mut triggers_vec) = triggers {
        let original_len = triggers_vec.len();
        triggers_vec.retain(|t| !t.trim().is_empty());
        if triggers_vec.len() < original_len {
            warnings.push(anyhow!(
                "empty triggers are not allowed and have been ignored"
            ));
        }
        if triggers_vec.is_empty() {
            None
        } else {
            Some(triggers_vec)
        }
    } else {
        None
    };

    // Resolve triggermarker configuration (3-level precedence)
    let (global_triggermarker_prefix, global_triggermarker_suffix, global_replace_mode,
         global_prefix_mode, global_suffix_mode, global_smart_chars, global_remove_multiple) = if let Some(cfg) = config {
        (
            cfg.triggermarker_prefix(),
            cfg.triggermarker_suffix(),
            cfg.triggermarker_replace_mode(),
            cfg.triggermarker_prefix_replace_mode(),
            cfg.triggermarker_suffix_replace_mode(),
            cfg.triggermarker_smart_chars(),
            cfg.triggermarker_smart_remove_multiple(),
        )
    } else {
        (None, None, "agnostic".to_string(), None, None, vec![], false)
    };

    // Match → defaults → global
    let triggermarker_prefix = yaml_match
        .triggermarker_prefix
        .or(defaults.and_then(|d| d.triggermarker_prefix.clone()))
        .or(global_triggermarker_prefix);

    let triggermarker_suffix = yaml_match
        .triggermarker_suffix
        .or(defaults.and_then(|d| d.triggermarker_suffix.clone()))
        .or(global_triggermarker_suffix);

    let base_replace_mode = yaml_match
        .triggermarker_replace_mode
        .or(defaults.and_then(|d| d.triggermarker_replace_mode.clone()))
        .unwrap_or(global_replace_mode);

    let prefix_replace_mode = yaml_match
        .triggermarker_prefix_replace_mode
        .or(defaults.and_then(|d| d.triggermarker_prefix_replace_mode.clone()))
        .or(global_prefix_mode)
        .unwrap_or(base_replace_mode.clone());

    let suffix_replace_mode = yaml_match
        .triggermarker_suffix_replace_mode
        .or(defaults.and_then(|d| d.triggermarker_suffix_replace_mode.clone()))
        .or(global_suffix_mode)
        .unwrap_or(base_replace_mode);

    let smart_chars = yaml_match
        .triggermarker_smart_chars
        .or(defaults.and_then(|d| d.triggermarker_smart_chars.clone()))
        .unwrap_or(global_smart_chars);

    let remove_multiple = yaml_match
        .triggermarker_smart_remove_multiple
        .or(defaults.and_then(|d| d.triggermarker_smart_remove_multiple))
        .unwrap_or(global_remove_multiple);

    // Validate triggermarker configuration
    if let Some(ref prefix) = triggermarker_prefix {
        if !prefix.is_empty() && prefix.chars().any(|c| c.is_alphanumeric()) {
            return Err(anyhow!(
                "Match validation error: triggermarker_prefix must not contain alphanumeric characters. Got: '{}'",
                prefix
            ));
        }
    }

    if let Some(ref suffix) = triggermarker_suffix {
        if !suffix.is_empty() && suffix.chars().any(|c| c.is_alphanumeric()) {
            return Err(anyhow!(
                "Match validation error: triggermarker_suffix must not contain alphanumeric characters. Got: '{}'",
                suffix
            ));
        }
    }

    if prefix_replace_mode != "agnostic" && prefix_replace_mode != "smart" {
        return Err(anyhow!(
            "Invalid triggermarker_prefix_replace_mode: '{}'. Must be 'agnostic' or 'smart'",
            prefix_replace_mode
        ));
    }

    if suffix_replace_mode != "agnostic" && suffix_replace_mode != "smart" {
        return Err(anyhow!(
            "Invalid triggermarker_suffix_replace_mode: '{}'. Must be 'agnostic' or 'smart'",
            suffix_replace_mode
        ));
    }

    // Apply triggermarkers to all triggers
    let triggers = if let Some(triggers) = triggers {
        Some(
            triggers
                .into_iter()
                .map(|trigger| {
                    apply_triggermarkers(
                        &trigger,
                        triggermarker_prefix.as_deref(),
                        triggermarker_suffix.as_deref(),
                        &prefix_replace_mode,
                        &suffix_replace_mode,
                        &smart_chars,
                        remove_multiple,
                    )
                })
                .collect()
        )
    } else {
        None
    };

    let uppercase_style = match yaml_match
        .uppercase_style
        .or(defaults.and_then(|d| d.uppercase_style.clone()))
        .map(|s| s.to_lowercase())
        .as_deref()
    {
        Some("uppercase") => UpperCasingStyle::Uppercase,
        Some("capitalize") => UpperCasingStyle::Capitalize,
        Some("capitalize_words") => UpperCasingStyle::CapitalizeWords,
        Some(style) => {
            warnings.push(anyhow!(
                "unrecognized uppercase_style: {:?}, falling back to the default",
                style
            ));
            TriggerCause::default().uppercase_style
        }
        _ => TriggerCause::default().uppercase_style,
    };

    let cause = if let Some(triggers) = triggers {
        MatchCause::Trigger(TriggerCause {
            triggers,
            left_word: yaml_match
                .left_word
                .or(yaml_match.word)
                .or(defaults.and_then(|d| d.left_word))
                .or(defaults.and_then(|d| d.word))
                .unwrap_or(TriggerCause::default().left_word),
            right_word: yaml_match
                .right_word
                .or(yaml_match.word)
                .or(defaults.and_then(|d| d.right_word))
                .or(defaults.and_then(|d| d.word))
                .unwrap_or(TriggerCause::default().right_word),
            propagate_case: yaml_match
                .propagate_case
                .or(defaults.and_then(|d| d.propagate_case))
                .unwrap_or(TriggerCause::default().propagate_case),
            uppercase_style,
        })
    } else if let Some(regex) = yaml_match.regex {
        MatchCause::Regex(RegexCause { regex })
    } else {
        MatchCause::None
    };

    let force_mode = if yaml_match.force_clipboard == Some(true) {
        Some(TextInjectMode::Clipboard)
    } else if let Some(mode) = yaml_match.force_mode {
        match mode.to_lowercase().as_str() {
            "clipboard" => Some(TextInjectMode::Clipboard),
            "keys" => Some(TextInjectMode::Keys),
            _ => None,
        }
    } else if defaults.and_then(|d| d.force_clipboard) == Some(true) {
        Some(TextInjectMode::Clipboard)
    } else if let Some(mode) = defaults.and_then(|d| d.force_mode.clone()) {
        match mode.to_lowercase().as_str() {
            "clipboard" => Some(TextInjectMode::Clipboard),
            "keys" => Some(TextInjectMode::Keys),
            _ => None,
        }
    } else {
        None
    };

    let effect = if yaml_match.replace.is_some()
        || yaml_match.markdown.is_some()
        || yaml_match.html.is_some()
    {
        let (replace, format) = if let Some(plain) = yaml_match.replace {
            (plain, TextFormat::Plain)
        } else if let Some(markdown) = yaml_match.markdown {
            (markdown, TextFormat::Markdown)
        } else if let Some(html) = yaml_match.html {
            (html, TextFormat::Html)
        } else {
            unreachable!();
        };

        let mut vars: Vec<Variable> = Vec::new();
        for yaml_var in yaml_match.vars.unwrap_or_default() {
            let (var, var_warnings) =
                try_convert_into_variable(yaml_var.clone(), use_compatibility_mode)
                    .with_context(|| format!("failed to load variable: {yaml_var:?}"))?;
            warnings.extend(var_warnings);
            vars.push(var);
        }

        MatchEffect::Text(TextEffect {
            replace,
            vars,
            format,
            force_mode,
        })
    } else if let Some(form_layout) = yaml_match.form {
        // Replace all the form fields with actual variables

        // In v2.1.0-alpha the form control syntax was replaced with [[control]]
        // instead of {{control}}, so we check if compatibility mode is being used.
        // TODO: remove once compatibility mode is removed

        let (resolved_replace, resolved_layout) = if use_compatibility_mode {
            (
                VAR_REGEX
                    .replace_all(&form_layout, |caps: &Captures| {
                        let var_name = caps.get(1).unwrap().as_str();
                        format!("{{{{form1.{var_name}}}}}")
                    })
                    .to_string(),
                VAR_REGEX
                    .replace_all(&form_layout, |caps: &Captures| {
                        let var_name = caps.get(1).unwrap().as_str();
                        format!("[[{var_name}]]")
                    })
                    .to_string(),
            )
        } else {
            (
                FORM_CONTROL_REGEX
                    .replace_all(&form_layout, |caps: &Captures| {
                        let var_name = caps.get(1).unwrap().as_str();
                        format!("{{{{form1.{var_name}}}}}")
                    })
                    .to_string(),
                form_layout,
            )
        };

        // Convert escaped brakets in forms
        let resolved_replace = resolved_replace.replace("\\{", "{ ").replace("\\}", " }");

        // Convert the form data to valid variables
        let mut params = Params::new();
        params.insert("layout".to_string(), Value::String(resolved_layout));

        if let Some(fields) = yaml_match.form_fields {
            params.insert("fields".to_string(), Value::Object(convert_params(fields)?));
        }

        let vars = vec![Variable {
            id: next_id(),
            name: "form1".to_owned(),
            var_type: "form".to_owned(),
            params,
            ..Default::default()
        }];

        MatchEffect::Text(TextEffect {
            replace: resolved_replace,
            vars,
            format: TextFormat::Plain,
            force_mode,
        })
    } else if let Some(image_path) = yaml_match.image_path {
        MatchEffect::Image(ImageEffect { path: image_path })
    } else {
        MatchEffect::None
    };

    if effect == MatchEffect::None {
        bail!(
      "match triggered by {:?} does not produce any effect. Did you forget the 'replace' field?",
      cause.long_description()
    );
    }

    Ok((
        Match {
            cause,
            effect,
            label: yaml_match.label,
            id: next_id(),
            search_terms: yaml_match.search_terms.unwrap_or_default(),
        },
        warnings,
    ))
}

pub fn try_convert_into_variable(
    yaml_var: YAMLVariable,
    use_compatibility_mode: bool,
) -> Result<(Variable, Vec<Warning>)> {
    Ok((
        Variable {
            name: yaml_var.name,
            var_type: yaml_var.var_type,
            params: convert_params(yaml_var.params)?,
            id: next_id(),
            inject_vars: !use_compatibility_mode && yaml_var.inject_vars.unwrap_or(true),
            depends_on: yaml_var.depends_on,
        },
        Vec::new(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        matches::{Match, Params, Value},
        util::tests::use_test_directory,
    };
    use std::fs::create_dir_all;

    fn create_match_with_warnings(
        yaml: &str,
        use_compatibility_mode: bool,
    ) -> Result<(Match, Vec<Warning>)> {
        let yaml_match: YAMLMatch = serde_norway::from_str(yaml)?;
        let (mut m, warnings) = try_convert_into_match(yaml_match, use_compatibility_mode, None, None)?;

        // Reset the IDs to correctly compare them
        m.id = 0;
        if let MatchEffect::Text(e) = &mut m.effect {
            e.vars.iter_mut().for_each(|v| v.id = 0);
        }

        Ok((m, warnings))
    }

    fn create_match(yaml: &str) -> Result<Match> {
        let (m, warnings) = create_match_with_warnings(yaml, false)?;
        assert!(
            warnings.is_empty(),
            "warnings were detected but not handled: {warnings:?}"
        );
        Ok(m)
    }

    #[test]
    fn basic_match_maps_correctly() {
        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "world".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn multiple_triggers_maps_correctly() {
        assert_eq!(
            create_match(
                r#"
        triggers: ["Hello", "john"]
        replace: "world"
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string(), "john".to_string()],
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "world".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn word_maps_correctly() {
        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        word: true
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    left_word: true,
                    right_word: true,
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "world".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn left_word_maps_correctly() {
        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        left_word: true
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    left_word: true,
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "world".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn right_word_maps_correctly() {
        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        right_word: true
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    right_word: true,
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "world".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn propagate_case_maps_correctly() {
        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        propagate_case: true
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    propagate_case: true,
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "world".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn uppercase_style_maps_correctly() {
        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        uppercase_style: "capitalize"
        propagate_case: true
        "#
            )
            .unwrap()
            .cause
            .into_trigger()
            .unwrap()
            .uppercase_style,
            UpperCasingStyle::Capitalize,
        );

        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        uppercase_style: "capitalize_words"
        propagate_case: true
        "#
            )
            .unwrap()
            .cause
            .into_trigger()
            .unwrap()
            .uppercase_style,
            UpperCasingStyle::CapitalizeWords,
        );

        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        uppercase_style: "uppercase"
        propagate_case: true
        "#
            )
            .unwrap()
            .cause
            .into_trigger()
            .unwrap()
            .uppercase_style,
            UpperCasingStyle::Uppercase,
        );

        // Invalid without propagate_case
        let (m, warnings) = create_match_with_warnings(
            r#"
        trigger: "Hello"
        replace: "world"
        uppercase_style: "capitalize"
        "#,
            false,
        )
        .unwrap();
        assert_eq!(
            m.cause.into_trigger().unwrap().uppercase_style,
            UpperCasingStyle::Capitalize,
        );
        assert_eq!(warnings.len(), 1);

        // Invalid style
        let (m, warnings) = create_match_with_warnings(
            r#"
        trigger: "Hello"
        replace: "world"
        uppercase_style: "invalid"
        propagate_case: true
        "#,
            false,
        )
        .unwrap();
        assert_eq!(
            m.cause.into_trigger().unwrap().uppercase_style,
            UpperCasingStyle::Uppercase,
        );
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn form_maps_correctly() {
        let mut params = Params::new();
        params.insert(
            "layout".to_string(),
            Value::String("Hi [[name]]!".to_string()),
        );

        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        form: "Hi [[name]]!"
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "Hi {{form1.name}}!".to_string(),
                    vars: vec![Variable {
                        id: 0,
                        name: "form1".to_string(),
                        var_type: "form".to_string(),
                        params,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn form_maps_correctly_with_variable_injection() {
        let mut params = Params::new();
        params.insert(
            "layout".to_string(),
            Value::String("Hi [[name]]! {{signature}}".to_string()),
        );

        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        form: "Hi [[name]]! {{signature}}"
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "Hi {{form1.name}}! {{signature}}".to_string(),
                    vars: vec![Variable {
                        id: 0,
                        name: "form1".to_string(),
                        var_type: "form".to_string(),
                        params,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn form_maps_correctly_legacy_format() {
        let mut params = Params::new();
        params.insert(
            "layout".to_string(),
            Value::String("Hi [[name]]!".to_string()),
        );

        assert_eq!(
            create_match_with_warnings(
                r#"
        trigger: "Hello"
        form: "Hi {{name}}!"
        "#,
                true
            )
            .unwrap()
            .0,
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "Hi {{form1.name}}!".to_string(),
                    vars: vec![Variable {
                        id: 0,
                        name: "form1".to_string(),
                        var_type: "form".to_string(),
                        params,
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn vars_maps_correctly() {
        let mut params = Params::new();
        params.insert("param1".to_string(), Value::Bool(true));
        let vars = vec![Variable {
            name: "var1".to_string(),
            var_type: "test".to_string(),
            params,
            ..Default::default()
        }];
        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        vars:
          - name: var1
            type: test
            params:
              param1: true
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "world".to_string(),
                    vars,
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn vars_inject_vars_and_depends_on() {
        let vars = vec![
            Variable {
                name: "var1".to_string(),
                var_type: "test".to_string(),
                depends_on: vec!["test".to_owned()],
                ..Default::default()
            },
            Variable {
                name: "var2".to_string(),
                var_type: "test".to_string(),
                inject_vars: false,
                ..Default::default()
            },
        ];
        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        vars:
          - name: var1
            type: test
            depends_on: ["test"]
          - name: var2
            type: "test"
            inject_vars: false
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "world".to_string(),
                    vars,
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn vars_no_params_maps_correctly() {
        let vars = vec![Variable {
            name: "var1".to_string(),
            var_type: "test".to_string(),
            params: Params::new(),
            ..Default::default()
        }];
        assert_eq!(
            create_match(
                r#"
        trigger: "Hello"
        replace: "world"
        vars:
          - name: var1
            type: test
        "#
            )
            .unwrap(),
            Match {
                cause: MatchCause::Trigger(TriggerCause {
                    triggers: vec!["Hello".to_string()],
                    ..Default::default()
                }),
                effect: MatchEffect::Text(TextEffect {
                    replace: "world".to_string(),
                    vars,
                    ..Default::default()
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn importer_is_supported() {
        let importer = YAMLImporter::new();
        assert!(importer.is_supported("yaml"));
        assert!(importer.is_supported("yml"));
        assert!(!importer.is_supported("invalid"));
    }

    #[test]
    fn importer_works_correctly() {
        use_test_directory(|_, match_dir, _| {
            let sub_dir = match_dir.join("sub");
            create_dir_all(&sub_dir).unwrap();

            let base_file = match_dir.join("base.yml");
            std::fs::write(
                &base_file,
                r#"
      imports:
        - "sub/sub.yml"
        - "invalid/import.yml" # This should be discarded
      
      global_vars:
        - name: "var1"
          type: "test"
      
      matches:
        - trigger: "hello"
          replace: "world"
      "#,
            )
            .unwrap();

            let sub_file = sub_dir.join("sub.yml");
            std::fs::write(&sub_file, "").unwrap();

            let importer = YAMLImporter::new();
            let (mut group, non_fatal_error_set) = importer.load_group(&base_file).unwrap();
            // The invalid import path should be reported as error
            assert_eq!(non_fatal_error_set.unwrap().errors.len(), 1);

            // Reset the ids to compare them correctly
            group.matches.iter_mut().for_each(|m| m.id = 0);
            group.global_vars.iter_mut().for_each(|v| v.id = 0);

            let vars = vec![Variable {
                name: "var1".to_string(),
                var_type: "test".to_string(),
                params: Params::new(),
                ..Default::default()
            }];

            assert_eq!(
                group,
                MatchGroup {
                    imports: vec![sub_file.to_string_lossy().to_string(),],
                    global_vars: vars,
                    matches: vec![Match {
                        cause: MatchCause::Trigger(TriggerCause {
                            triggers: vec!["hello".to_string()],
                            ..Default::default()
                        }),
                        effect: MatchEffect::Text(TextEffect {
                            replace: "world".to_string(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                }
            );
        });
    }

    #[test]
    fn importer_invalid_syntax() {
        use_test_directory(|_, match_dir, _| {
            let base_file = match_dir.join("base.yml");
            std::fs::write(
                &base_file,
                r"
      imports:
        - invalid
       - indentation
      ",
            )
            .unwrap();

            let importer = YAMLImporter::new();
            assert!(importer.load_group(&base_file).is_err());
        });
    }

    #[test]
    fn match_defaults_are_applied() {
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
match_defaults:
  propagate_case: true
  word: true
matches:
  - trigger: "test"
    replace: "replacement"
"#,
        )
        .unwrap();

        let yaml_match = &yaml_group.matches.unwrap()[0];
        let (m, _) = try_convert_into_match(
            yaml_match.clone(),
            false,
            yaml_group.match_defaults.as_ref(),
            None,
        )
        .unwrap();

        if let MatchCause::Trigger(cause) = m.cause {
            assert!(cause.propagate_case);
            assert!(cause.left_word);
            assert!(cause.right_word);
        } else {
            panic!("Expected TriggerCause");
        }
    }

    #[test]
    fn match_options_override_defaults() {
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
match_defaults:
  propagate_case: true
  word: true
matches:
  - trigger: "test"
    replace: "replacement"
    propagate_case: false
    left_word: false
"#,
        )
        .unwrap();

        let yaml_match = &yaml_group.matches.unwrap()[0];
        let (m, _) = try_convert_into_match(
            yaml_match.clone(),
            false,
            yaml_group.match_defaults.as_ref(),
            None,
        )
        .unwrap();

        if let MatchCause::Trigger(cause) = m.cause {
            assert!(!cause.propagate_case);
            assert!(!cause.left_word);
            assert!(cause.right_word); // Still from default
        } else {
            panic!("Expected TriggerCause");
        }
    }

    #[test]
    fn match_defaults_uppercase_style() {
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
match_defaults:
  propagate_case: true
  uppercase_style: "capitalize"
matches:
  - trigger: "test"
    replace: "replacement"
"#,
        )
        .unwrap();

        let yaml_match = &yaml_group.matches.unwrap()[0];
        let (m, _) = try_convert_into_match(
            yaml_match.clone(),
            false,
            yaml_group.match_defaults.as_ref(),
            None,
        )
        .unwrap();

        if let MatchCause::Trigger(cause) = m.cause {
            assert!(cause.propagate_case);
            assert_eq!(cause.uppercase_style, UpperCasingStyle::Capitalize);
        } else {
            panic!("Expected TriggerCause");
        }
    }

    #[test]
    fn match_defaults_force_mode() {
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
match_defaults:
  force_clipboard: true
matches:
  - trigger: "test"
    replace: "replacement"
"#,
        )
        .unwrap();

        let yaml_match = &yaml_group.matches.unwrap()[0];
        let (m, _) = try_convert_into_match(
            yaml_match.clone(),
            false,
            yaml_group.match_defaults.as_ref(),
            None,
        )
        .unwrap();

        if let MatchEffect::Text(effect) = m.effect {
            assert_eq!(effect.force_mode, Some(TextInjectMode::Clipboard));
        } else {
            panic!("Expected TextEffect");
        }
    }

    #[test]
    fn no_match_defaults_works_as_before() {
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
matches:
  - trigger: "test"
    replace: "replacement"
"#,
        )
        .unwrap();

        let yaml_match = &yaml_group.matches.unwrap()[0];
        let (m, _) = try_convert_into_match(yaml_match.clone(), false, None, None).unwrap();

        if let MatchCause::Trigger(cause) = m.cause {
            assert!(!cause.propagate_case);
            assert!(!cause.left_word);
            assert!(!cause.right_word);
        } else {
            panic!("Expected TriggerCause");
        }
    }

    #[test]
    fn match_force_mode_overrides_default_force_clipboard() {
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
match_defaults:
  force_clipboard: true
matches:
  - trigger: "test"
    replace: "replacement"
    force_mode: "keys"
"#,
        )
        .unwrap();

        let yaml_match = &yaml_group.matches.unwrap()[0];
        let (m, _) = try_convert_into_match(
            yaml_match.clone(),
            false,
            yaml_group.match_defaults.as_ref(),
            None,
        )
        .unwrap();

        if let MatchEffect::Text(effect) = m.effect {
            assert_eq!(effect.force_mode, Some(TextInjectMode::Keys));
        } else {
            panic!("Expected TextEffect");
        }
    }

    #[test]
    fn uppercase_style_warning_with_defaults() {
        // Warning when uppercase_style in defaults but propagate_case is false
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
match_defaults:
  uppercase_style: "capitalize"
matches:
  - trigger: "test"
    replace: "replacement"
    propagate_case: false
"#,
        )
        .unwrap();

        let yaml_match = &yaml_group.matches.unwrap()[0];
        let (_, warnings) = try_convert_into_match(
            yaml_match.clone(),
            false,
            yaml_group.match_defaults.as_ref(),
            None,
        )
        .unwrap();
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn regex_matches_not_affected_by_trigger_defaults() {
        // Regex matches should not be affected by trigger-specific defaults
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
match_defaults:
  word: true
  propagate_case: true
matches:
  - regex: "test\\d+"
    replace: "matched"
"#,
        )
        .unwrap();

        let yaml_match = &yaml_group.matches.unwrap()[0];
        let (m, _) = try_convert_into_match(
            yaml_match.clone(),
            false,
            yaml_group.match_defaults.as_ref(),
            None,
        )
        .unwrap();

        // Regex matches should have RegexCause, not TriggerCause
        assert!(matches!(m.cause, MatchCause::Regex(_)));
    }

    #[test]
    fn test_match_defaults_propagate_case() {
        // Test that propagate_case is correctly inherited from defaults
        let yaml_group: YAMLMatchGroup = serde_norway::from_str(
            r#"
match_defaults:
  propagate_case: true
matches:
  - trigger: ":test"
    replace: "replacement"
"#,
        )
        .unwrap();

        let yaml_match = &yaml_group.matches.unwrap()[0];
        let (m, _) = try_convert_into_match(
            yaml_match.clone(),
            false,
            yaml_group.match_defaults.as_ref(),
            None,
        )
        .unwrap();

        // Verify the match has propagate_case set to true from defaults
        if let MatchCause::Trigger(cause) = m.cause {
            assert!(
                cause.propagate_case,
                "propagate_case should be true from defaults"
            );
        } else {
            panic!("Expected TriggerCause");
        }
    }
}

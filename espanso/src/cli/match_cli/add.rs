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

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use serde_norway::Value;

use crate::cli::edit::determine_target_path;

#[derive(Debug)]
pub struct NewMatch {
    pub trigger: String,
    pub replace: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct YAMLMatchFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    imports: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    global_vars: Option<Vec<Value>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    matches: Option<Vec<YAMLMatch>>,

    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct YAMLMatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    trigger: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    triggers: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    replace: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    label: Option<String>,

    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

pub fn add_main(cli_args: &ArgMatches, config_path: &Path) -> Result<()> {
    let trigger = cli_args
        .value_of("trigger")
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("missing required option --trigger"))?;
    let replace = cli_args
        .value_of("replace")
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("missing required option --replace"))?;
    let label = cli_args.value_of("label").map(str::to_owned);
    let target_file = cli_args.value_of("target_file");

    let target_path = determine_target_path(config_path, target_file);
    let new_match = NewMatch {
        trigger,
        replace,
        label,
    };

    add_match_to_yaml_file(&target_path, &new_match)?;

    println!(
        "Added match '{}' to {}",
        new_match.trigger,
        target_path.display(),
    );

    Ok(())
}

pub fn add_match_to_yaml_file(target_path: &Path, new_match: &NewMatch) -> Result<()> {
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("unable to create directory {}", parent.display()))?;
    }

    let content = if target_path.exists() {
        Some(
            std::fs::read_to_string(target_path)
                .with_context(|| format!("unable to read file {}", target_path.display()))?,
        )
    } else {
        None
    };

    let mut match_file: YAMLMatchFile = if let Some(content) = content {
        YAMLMatchFile::parse_from_str(&content)
            .with_context(|| format!("unable to parse YAML file {}", target_path.display()))?
    } else {
        YAMLMatchFile::default()
    };

    let existing_matches = match_file.matches.get_or_insert_with(Vec::new);
    if existing_matches
        .iter()
        .any(|existing| has_trigger(existing, &new_match.trigger))
    {
        bail!(
            "the trigger '{}' already exists in {}",
            new_match.trigger,
            target_path.display()
        );
    }

    existing_matches.push(YAMLMatch {
        trigger: Some(new_match.trigger.clone()),
        replace: Some(new_match.replace.clone()),
        label: new_match.label.clone(),
        ..Default::default()
    });

    let serialized = serde_norway::to_string(&match_file)
        .context("unable to serialize updated match file as YAML")?;
    std::fs::write(target_path, serialized)
        .with_context(|| format!("unable to write file {}", target_path.display()))?;

    Ok(())
}

fn has_trigger(existing: &YAMLMatch, trigger: &str) -> bool {
    if existing.trigger.as_deref() == Some(trigger) {
        return true;
    }

    existing
        .triggers
        .as_ref()
        .is_some_and(|triggers| triggers.iter().any(|existing| existing == trigger))
}

impl YAMLMatchFile {
    fn parse_from_str(yaml: &str) -> Result<Self> {
        // Remove UTF-8 BOM if present (common in Windows editors like Notepad)
        let yaml = yaml.trim_start_matches('\u{FEFF}');

        if yaml.trim().is_empty() {
            return Ok(Self::default());
        }

        Ok(serde_norway::from_str(yaml)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn add_match_to_existing_file() {
        let temp_dir = TempDir::new("espanso_match_add").unwrap();
        let file_path = temp_dir.path().join("base.yml");
        std::fs::write(
            &file_path,
            r#"
matches:
  - trigger: ":hello"
    replace: "Hi"
"#,
        )
        .unwrap();

        add_match_to_yaml_file(
            &file_path,
            &NewMatch {
                trigger: ":bye".to_string(),
                replace: "Goodbye".to_string(),
                label: Some("Farewell".to_string()),
            },
        )
        .unwrap();

        let output = std::fs::read_to_string(file_path).unwrap();
        let parsed: YAMLMatchFile = serde_norway::from_str(&output).unwrap();
        let matches = parsed.matches.unwrap();

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].trigger.as_deref(), Some(":hello"));
        assert_eq!(matches[1].trigger.as_deref(), Some(":bye"));
        assert_eq!(matches[1].replace.as_deref(), Some("Goodbye"));
        assert_eq!(matches[1].label.as_deref(), Some("Farewell"));
    }

    #[test]
    fn rejects_duplicate_trigger() {
        let temp_dir = TempDir::new("espanso_match_add").unwrap();
        let file_path = temp_dir.path().join("base.yml");
        std::fs::write(
            &file_path,
            r#"
matches:
  - triggers: [":hello", ":hola"]
    replace: "Hi"
"#,
        )
        .unwrap();

        let result = add_match_to_yaml_file(
            &file_path,
            &NewMatch {
                trigger: ":hola".to_string(),
                replace: "Hello".to_string(),
                label: None,
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn creates_missing_match_file() {
        let temp_dir = TempDir::new("espanso_match_add").unwrap();
        let file_path = temp_dir.path().join("nested").join("base.yml");

        add_match_to_yaml_file(
            &file_path,
            &NewMatch {
                trigger: ":hello".to_string(),
                replace: "Hi".to_string(),
                label: None,
            },
        )
        .unwrap();

        let output = std::fs::read_to_string(file_path).unwrap();
        let parsed: YAMLMatchFile = serde_norway::from_str(&output).unwrap();
        let matches = parsed.matches.unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].trigger.as_deref(), Some(":hello"));
        assert_eq!(matches[0].replace.as_deref(), Some("Hi"));
    }
}

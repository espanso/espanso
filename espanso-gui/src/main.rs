use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_norway::Value;

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

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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

#[derive(Debug, Serialize)]
struct MatchRecord {
    id: String,
    trigger: String,
    replace: String,
    label: Option<String>,
    has_multiple_triggers: bool,
}

#[derive(Debug, Deserialize)]
struct AddMatchRequest {
    file_path: String,
    trigger: String,
    replace: String,
    label: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateMatchRequest {
    file_path: String,
    id: String,
    trigger: String,
    replace: String,
    label: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RemoveMatchRequest {
    file_path: String,
    id: String,
}

#[tauri::command]
fn get_default_match_file() -> std::result::Result<String, String> {
    default_match_file()
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn list_matches(file_path: String) -> std::result::Result<Vec<MatchRecord>, String> {
    list_matches_impl(Path::new(&file_path)).map_err(|err| err.to_string())
}

#[tauri::command]
fn add_match(payload: AddMatchRequest) -> std::result::Result<(), String> {
    add_match_impl(payload).map_err(|err| err.to_string())
}

#[tauri::command]
fn update_match(payload: UpdateMatchRequest) -> std::result::Result<(), String> {
    update_match_impl(payload).map_err(|err| err.to_string())
}

#[tauri::command]
fn remove_match(payload: RemoveMatchRequest) -> std::result::Result<(), String> {
    remove_match_impl(payload).map_err(|err| err.to_string())
}

fn default_match_file() -> Result<PathBuf> {
    if let Some(config_dir) = std::env::var_os("ESPANSO_CONFIG_DIR") {
        return Ok(PathBuf::from(config_dir).join("match").join("base.yml"));
    }

    let config_root = dirs::config_dir().context("unable to resolve user config directory")?;
    Ok(config_root.join("espanso").join("match").join("base.yml"))
}

fn parse_yaml_match_file(content: &str) -> Result<YAMLMatchFile> {
    let content = content.trim_start_matches('\u{FEFF}');
    if content.trim().is_empty() {
        return Ok(YAMLMatchFile::default());
    }

    Ok(serde_norway::from_str(content)?)
}

fn load_yaml_match_file(target_path: &Path) -> Result<YAMLMatchFile> {
    if !target_path.exists() {
        return Ok(YAMLMatchFile {
            matches: Some(Vec::new()),
            ..Default::default()
        });
    }

    let content = std::fs::read_to_string(target_path)
        .with_context(|| format!("unable to read file {}", target_path.display()))?;
    parse_yaml_match_file(&content)
        .with_context(|| format!("unable to parse YAML file {}", target_path.display()))
}

fn save_yaml_match_file(target_path: &Path, file: &YAMLMatchFile) -> Result<()> {
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("unable to create directory {}", parent.display()))?;
    }

    let serialized =
        serde_norway::to_string(file).context("unable to serialize updated YAML match file")?;

    std::fs::write(target_path, serialized)
        .with_context(|| format!("unable to write file {}", target_path.display()))?;
    Ok(())
}

fn extract_primary_trigger(entry: &YAMLMatch) -> Option<String> {
    if let Some(trigger) = &entry.trigger {
        return Some(trigger.clone());
    }

    entry.triggers.as_ref().and_then(|v| v.first().cloned())
}

fn all_triggers(entry: &YAMLMatch) -> Vec<String> {
    let mut results = Vec::new();
    if let Some(trigger) = &entry.trigger {
        results.push(trigger.clone());
    }
    if let Some(triggers) = &entry.triggers {
        results.extend(triggers.clone());
    }
    results
}

fn normalize_label(label: Option<String>) -> Option<String> {
    label.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn normalize_non_empty(value: &str, field_name: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("{} cannot be empty", field_name);
    }
    Ok(trimmed.to_string())
}

fn validate_unique_trigger(
    matches: &[YAMLMatch],
    new_trigger: &str,
    ignore_index: Option<usize>,
) -> Result<()> {
    let duplicate = matches.iter().enumerate().any(|(idx, existing)| {
        if ignore_index == Some(idx) {
            return false;
        }

        all_triggers(existing)
            .iter()
            .any(|existing| existing == new_trigger)
    });

    if duplicate {
        bail!("trigger '{new_trigger}' already exists in this file");
    }

    Ok(())
}

fn parse_match_index(id: &str, matches_len: usize) -> Result<usize> {
    let index = id
        .parse::<usize>()
        .with_context(|| format!("invalid match id '{id}'"))?;

    if index >= matches_len {
        bail!("match id '{id}' does not exist");
    }

    Ok(index)
}

fn list_matches_impl(target_path: &Path) -> Result<Vec<MatchRecord>> {
    let file = load_yaml_match_file(target_path)?;
    let mut records = Vec::new();
    if let Some(matches) = file.matches {
        for (idx, entry) in matches.iter().enumerate() {
            if let Some(primary_trigger) = extract_primary_trigger(entry) {
                records.push(MatchRecord {
                    id: idx.to_string(),
                    trigger: primary_trigger,
                    replace: entry.replace.clone().unwrap_or_default(),
                    label: normalize_label(entry.label.clone()),
                    has_multiple_triggers: entry
                        .triggers
                        .as_ref()
                        .is_some_and(|triggers| triggers.len() > 1),
                });
            }
        }
    }

    Ok(records)
}

fn add_match_impl(payload: AddMatchRequest) -> Result<()> {
    let target_path = PathBuf::from(payload.file_path);
    let trigger = normalize_non_empty(&payload.trigger, "trigger")?;
    let replace = normalize_non_empty(&payload.replace, "replace")?;
    let label = normalize_label(payload.label);

    let mut file = load_yaml_match_file(&target_path)?;
    let matches = file.matches.get_or_insert_with(Vec::new);

    validate_unique_trigger(matches, &trigger, None)?;

    matches.push(YAMLMatch {
        trigger: Some(trigger),
        triggers: None,
        replace: Some(replace),
        label,
        extra: BTreeMap::new(),
    });

    save_yaml_match_file(&target_path, &file)
}

fn update_match_impl(payload: UpdateMatchRequest) -> Result<()> {
    let target_path = PathBuf::from(payload.file_path);
    let trigger = normalize_non_empty(&payload.trigger, "trigger")?;
    let replace = normalize_non_empty(&payload.replace, "replace")?;
    let label = normalize_label(payload.label);

    let mut file = load_yaml_match_file(&target_path)?;
    let matches = file.matches.get_or_insert_with(Vec::new);
    let index = parse_match_index(&payload.id, matches.len())?;

    validate_unique_trigger(matches, &trigger, Some(index))?;

    let existing = matches
        .get_mut(index)
        .context("unable to update selected match")?;

    existing.trigger = Some(trigger);
    existing.triggers = None;
    existing.replace = Some(replace);
    existing.label = label;

    save_yaml_match_file(&target_path, &file)
}

fn remove_match_impl(payload: RemoveMatchRequest) -> Result<()> {
    let target_path = PathBuf::from(payload.file_path);

    let mut file = load_yaml_match_file(&target_path)?;
    let matches = file.matches.get_or_insert_with(Vec::new);
    let index = parse_match_index(&payload.id, matches.len())?;
    matches.remove(index);

    save_yaml_match_file(&target_path, &file)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_default_match_file,
            list_matches,
            add_match,
            update_match,
            remove_match
        ])
        .run(tauri::generate_context!())
        .expect("error while running Espanso Match Studio");
}

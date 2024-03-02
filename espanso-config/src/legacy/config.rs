// This file is taken from the old version of espanso, and used to load
// the "legacy" config format

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

use super::model::{KeyModifier, PasteShortcut};
use log::error;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::string::ToString;
use walkdir::{DirEntry, WalkDir};

pub const DEFAULT_CONFIG_FILE_NAME: &str = "default.yml";
pub const USER_CONFIGS_FOLDER_NAME: &str = "user";

// Default values for primitives
fn default_name() -> String {
  "default".to_owned()
}
fn default_parent() -> String {
  "self".to_owned()
}
fn default_filter_title() -> String {
  String::new()
}
fn default_filter_class() -> String {
  String::new()
}
fn default_filter_exec() -> String {
  String::new()
}
fn default_log_level() -> i32 {
  0
}
fn default_conflict_check() -> bool {
  false
}
fn default_ipc_server_port() -> i32 {
  34982
}
fn default_worker_ipc_server_port() -> i32 {
  34983
}
fn default_use_system_agent() -> bool {
  true
}
fn default_config_caching_interval() -> i32 {
  800
}
fn default_word_separators() -> Vec<char> {
  vec![' ', ',', '.', '?', '!', '\r', '\n', 22u8 as char]
}
fn default_toggle_interval() -> u32 {
  230
}
fn default_toggle_key() -> KeyModifier {
  KeyModifier::ALT
}
fn default_preserve_clipboard() -> bool {
  true
}
fn default_passive_match_regex() -> String {
  "(?P<name>:\\p{L}+)(/(?P<args>.*)/)?".to_owned()
}
fn default_passive_arg_delimiter() -> char {
  '/'
}
fn default_passive_arg_escape() -> char {
  '\\'
}
fn default_passive_delay() -> u64 {
  100
}
fn default_passive_key() -> KeyModifier {
  KeyModifier::OFF
}
fn default_enable_passive() -> bool {
  false
}
fn default_enable_active() -> bool {
  true
}
fn default_backspace_limit() -> i32 {
  3
}
fn default_backspace_delay() -> i32 {
  0
}
fn default_inject_delay() -> i32 {
  0
}
fn default_restore_clipboard_delay() -> i32 {
  300
}
fn default_exclude_default_entries() -> bool {
  false
}
fn default_secure_input_watcher_enabled() -> bool {
  true
}
fn default_secure_input_notification() -> bool {
  true
}
fn default_show_notifications() -> bool {
  true
}
fn default_auto_restart() -> bool {
  true
}
fn default_undo_backspace() -> bool {
  true
}
fn default_show_icon() -> bool {
  true
}
fn default_fast_inject() -> bool {
  true
}
fn default_secure_input_watcher_interval() -> i32 {
  5000
}
fn default_matches() -> Vec<Value> {
  Vec::new()
}
fn default_global_vars() -> Vec<Value> {
  Vec::new()
}
fn default_modulo_path() -> Option<String> {
  None
}
fn default_post_inject_delay() -> u64 {
  100
}
fn default_wait_for_modifiers_release() -> bool {
  false
}
fn default_search_trigger() -> Option<String> {
  Some("jkj".to_string())
}
fn default_search_shortcut() -> Option<String> {
  Some("ALT+SPACE".to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyConfig {
  #[serde(default = "default_name")]
  pub name: String,

  #[serde(default = "default_parent")]
  pub parent: String,

  #[serde(default = "default_filter_title")]
  pub filter_title: String,

  #[serde(default = "default_filter_class")]
  pub filter_class: String,

  #[serde(default = "default_filter_exec")]
  pub filter_exec: String,

  #[serde(default = "default_log_level")]
  pub log_level: i32,

  #[serde(default = "default_conflict_check")]
  pub conflict_check: bool,

  #[serde(default = "default_ipc_server_port")]
  pub ipc_server_port: i32,

  #[serde(default = "default_worker_ipc_server_port")]
  pub worker_ipc_server_port: i32,

  #[serde(default = "default_use_system_agent")]
  pub use_system_agent: bool,

  #[serde(default = "default_config_caching_interval")]
  pub config_caching_interval: i32,

  #[serde(default = "default_word_separators")]
  pub word_separators: Vec<char>,

  #[serde(default = "default_toggle_key")]
  pub toggle_key: KeyModifier,

  #[serde(default = "default_toggle_interval")]
  pub toggle_interval: u32,

  #[serde(default = "default_preserve_clipboard")]
  pub preserve_clipboard: bool,

  #[serde(default = "default_passive_match_regex")]
  pub passive_match_regex: String,

  #[serde(default = "default_passive_arg_delimiter")]
  pub passive_arg_delimiter: char,

  #[serde(default = "default_passive_arg_escape")]
  pub passive_arg_escape: char,

  #[serde(default = "default_passive_key")]
  pub passive_key: KeyModifier,

  #[serde(default = "default_passive_delay")]
  pub passive_delay: u64,

  #[serde(default = "default_enable_passive")]
  pub enable_passive: bool,

  #[serde(default = "default_enable_active")]
  pub enable_active: bool,

  #[serde(default = "default_undo_backspace")]
  pub undo_backspace: bool,

  #[serde(default)]
  pub paste_shortcut: PasteShortcut,

  #[serde(default = "default_backspace_limit")]
  pub backspace_limit: i32,

  #[serde(default = "default_restore_clipboard_delay")]
  pub restore_clipboard_delay: i32,

  #[serde(default = "default_secure_input_watcher_enabled")]
  pub secure_input_watcher_enabled: bool,

  #[serde(default = "default_secure_input_watcher_interval")]
  pub secure_input_watcher_interval: i32,

  #[serde(default = "default_post_inject_delay")]
  pub post_inject_delay: u64,

  #[serde(default = "default_secure_input_notification")]
  pub secure_input_notification: bool,

  #[serde(default)]
  pub backend: BackendType,

  #[serde(default = "default_exclude_default_entries")]
  pub exclude_default_entries: bool,

  #[serde(default = "default_show_notifications")]
  pub show_notifications: bool,

  #[serde(default = "default_show_icon")]
  pub show_icon: bool,

  #[serde(default = "default_fast_inject")]
  pub fast_inject: bool,

  #[serde(default = "default_backspace_delay")]
  pub backspace_delay: i32,

  #[serde(default = "default_inject_delay")]
  pub inject_delay: i32,

  #[serde(default = "default_auto_restart")]
  pub auto_restart: bool,

  #[serde(default = "default_matches")]
  pub matches: Vec<Value>,

  #[serde(default = "default_global_vars")]
  pub global_vars: Vec<Value>,

  #[serde(default = "default_modulo_path")]
  pub modulo_path: Option<String>,

  #[serde(default = "default_search_trigger")]
  pub search_trigger: Option<String>,

  #[serde(default = "default_search_shortcut")]
  pub search_shortcut: Option<String>,

  #[serde(default = "default_wait_for_modifiers_release")]
  pub wait_for_modifiers_release: bool,
}

// Macro used to validate config fields
#[macro_export]
macro_rules! validate_field {
    ($result:expr, $field:expr, $def_value:expr) => {
        if $field != $def_value {
            let mut field_name = stringify!($field);
            if field_name.starts_with("self.") {
                field_name = &field_name[5..];  // Remove the 'self.' prefix
            }
            error!("Validation error, parameter '{}' is reserved and can be only used in the default.yml config file", field_name);
            $result = false;
        }
    };
}

impl LegacyConfig {
  /*
   * Validate the Config instance.
   * It makes sure that user defined config instances do not define
   * attributes reserved to the default config.
   */
  fn validate_user_defined_config(&self) -> bool {
    let mut result = true;

    validate_field!(
      result,
      self.config_caching_interval,
      default_config_caching_interval()
    );
    validate_field!(result, self.log_level, default_log_level());
    validate_field!(result, self.conflict_check, default_conflict_check());
    validate_field!(result, self.toggle_key, default_toggle_key());
    validate_field!(result, self.toggle_interval, default_toggle_interval());
    validate_field!(result, self.backspace_limit, default_backspace_limit());
    validate_field!(result, self.ipc_server_port, default_ipc_server_port());
    validate_field!(result, self.use_system_agent, default_use_system_agent());
    validate_field!(
      result,
      self.preserve_clipboard,
      default_preserve_clipboard()
    );
    validate_field!(
      result,
      self.passive_match_regex,
      default_passive_match_regex()
    );
    validate_field!(
      result,
      self.passive_arg_delimiter,
      default_passive_arg_delimiter()
    );
    validate_field!(
      result,
      self.passive_arg_escape,
      default_passive_arg_escape()
    );
    validate_field!(result, self.passive_key, default_passive_key());
    validate_field!(
      result,
      self.restore_clipboard_delay,
      default_restore_clipboard_delay()
    );
    validate_field!(
      result,
      self.secure_input_watcher_enabled,
      default_secure_input_watcher_enabled()
    );
    validate_field!(
      result,
      self.secure_input_watcher_interval,
      default_secure_input_watcher_interval()
    );
    validate_field!(
      result,
      self.secure_input_notification,
      default_secure_input_notification()
    );
    validate_field!(
      result,
      self.show_notifications,
      default_show_notifications()
    );
    validate_field!(result, self.show_icon, default_show_icon());

    result
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Default)]
pub enum BackendType {
  Inject,
  Clipboard,
  #[default]
  Auto,
}

impl LegacyConfig {
  fn load_config(path: &Path) -> Result<LegacyConfig, ConfigLoadError> {
    let file_res = File::open(path);
    if let Ok(mut file) = file_res {
      let mut contents = String::new();
      let res = file.read_to_string(&mut contents);

      if res.is_err() {
        return Err(ConfigLoadError::UnableToReadFile);
      }

      let config_res = serde_yaml::from_str(&contents);

      match config_res {
        Ok(config) => Ok(config),
        Err(e) => Err(ConfigLoadError::InvalidYAML(path.to_owned(), e.to_string())),
      }
    } else {
      eprintln!("Error: Cannot load file {path:?}");
      Err(ConfigLoadError::FileNotFound)
    }
  }

  fn merge_overwrite(&mut self, new_config: LegacyConfig) {
    // Merge matches
    let mut merged_matches = new_config.matches;
    let mut match_trigger_set = HashSet::new();
    for m in &merged_matches {
      match_trigger_set.extend(triggers_for_match(m));
    }
    let parent_matches: Vec<Value> = self
      .matches
      .iter()
      .filter(|&m| {
        !triggers_for_match(m)
          .iter()
          .any(|trigger| match_trigger_set.contains(trigger))
      })
      .cloned()
      .collect();

    merged_matches.extend(parent_matches);
    self.matches = merged_matches;

    // Merge global variables
    let mut merged_global_vars = new_config.global_vars;
    let mut vars_name_set = HashSet::new();
    for m in &merged_global_vars {
      vars_name_set.insert(name_for_global_var(m));
    }
    let parent_vars: Vec<Value> = self
      .global_vars
      .iter()
      .filter(|&m| !vars_name_set.contains(&name_for_global_var(m)))
      .cloned()
      .collect();

    merged_global_vars.extend(parent_vars);
    self.global_vars = merged_global_vars;
  }

  fn merge_no_overwrite(&mut self, default: &LegacyConfig) {
    // Merge matches
    let mut match_trigger_set = HashSet::new();
    self.matches.iter().for_each(|m| {
      match_trigger_set.extend(triggers_for_match(m));
    });
    let default_matches: Vec<Value> = default
      .matches
      .iter()
      .filter(|&m| {
        !triggers_for_match(m)
          .iter()
          .any(|trigger| match_trigger_set.contains(trigger))
      })
      .cloned()
      .collect();

    self.matches.extend(default_matches);

    // Merge global variables
    let mut vars_name_set = HashSet::new();
    self.global_vars.iter().for_each(|m| {
      vars_name_set.insert(name_for_global_var(m));
    });
    let default_vars: Vec<Value> = default
      .global_vars
      .iter()
      .filter(|&m| !vars_name_set.contains(&name_for_global_var(m)))
      .cloned()
      .collect();

    self.global_vars.extend(default_vars);
  }
}

fn triggers_for_match(m: &Value) -> Vec<String> {
  if let Some(triggers) = m.get("triggers").and_then(|v| v.as_sequence()) {
    triggers
      .iter()
      .filter_map(|v| v.as_str().map(ToString::to_string))
      .collect()
  } else if let Some(trigger) = m.get("trigger").and_then(|v| v.as_str()) {
    vec![trigger.to_string()]
  } else {
    vec![]
  }
}

#[allow(dead_code)]
fn replace_for_match(m: &Value) -> String {
  m.get("replace")
    .and_then(|v| v.as_str())
    .expect("match is missing replace field")
    .to_string()
}

fn name_for_global_var(v: &Value) -> String {
  v.get("name")
    .and_then(|v| v.as_str())
    .expect("global var is missing name field")
    .to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyConfigSet {
  pub default: LegacyConfig,
  pub specific: Vec<LegacyConfig>,
}

impl LegacyConfigSet {
  pub fn load(config_dir: &Path, package_dir: &Path) -> Result<LegacyConfigSet, ConfigLoadError> {
    if !config_dir.is_dir() {
      return Err(ConfigLoadError::InvalidConfigDirectory);
    }

    // Load default configuration
    let default_file = config_dir.join(DEFAULT_CONFIG_FILE_NAME);
    let default = LegacyConfig::load_config(default_file.as_path())?;

    // Analyze which config files have to be loaded

    let mut target_files = Vec::new();

    let specific_dir = config_dir.join(USER_CONFIGS_FOLDER_NAME);
    if specific_dir.exists() {
      let dir_entry = WalkDir::new(specific_dir);
      target_files.extend(dir_entry);
    }

    let package_files = if package_dir.exists() {
      let dir_entry = WalkDir::new(package_dir);
      dir_entry.into_iter().collect()
    } else {
      Vec::new()
    };

    // Load the user defined config files

    let mut name_set = HashSet::new();
    let mut children_map: HashMap<String, Vec<LegacyConfig>> = HashMap::new();
    let mut package_map: HashMap<String, Vec<LegacyConfig>> = HashMap::new();
    let mut root_configs = vec![default];

    let mut file_loader = |entry: walkdir::Result<DirEntry>,
                           dest_map: &mut HashMap<String, Vec<LegacyConfig>>|
     -> Result<(), ConfigLoadError> {
      match entry {
        Ok(entry) => {
          let path = entry.path();

          // Skip non-yaml config files
          if path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            != "yml"
          {
            return Ok(());
          }

          // Skip hidden files
          if path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .starts_with('.')
          {
            return Ok(());
          }

          let mut config = LegacyConfig::load_config(path)?;

          // Make sure the config does not contain reserved fields
          if !config.validate_user_defined_config() {
            return Err(ConfigLoadError::InvalidParameter(path.to_owned()));
          }

          // No name specified, defaulting to the path name
          if config.name == "default" {
            config.name = path.to_str().unwrap_or_default().to_owned();
          }

          if name_set.contains(&config.name) {
            return Err(ConfigLoadError::NameDuplicate(path.to_owned()));
          }

          name_set.insert(config.name.clone());

          if config.parent == "self" {
            // No parent, root config
            root_configs.push(config);
          } else {
            // Children config
            let children_vec = dest_map.entry(config.parent.clone()).or_default();
            children_vec.push(config);
          }
        }
        Err(e) => {
          eprintln!("Warning: Unable to read config file: {e}");
        }
      }

      Ok(())
    };

    // Load the default and user specific configs
    for entry in target_files {
      file_loader(entry, &mut children_map)?;
    }

    // Load the package related configs
    for entry in package_files {
      file_loader(entry, &mut package_map)?;
    }

    // Merge the children config files
    let mut configs_without_packages = Vec::new();
    for root_config in root_configs {
      let config = LegacyConfigSet::reduce_configs(root_config, &children_map, true);
      configs_without_packages.push(config);
    }

    // Merge package files
    // Note: we need two different steps as the packages have a lower priority
    //       than configs.
    let mut configs = Vec::new();
    for root_config in configs_without_packages {
      let config = LegacyConfigSet::reduce_configs(root_config, &package_map, false);
      configs.push(config);
    }

    // Separate default from specific
    let default = configs.first().unwrap().clone();
    let mut specific = configs[1..].to_vec();

    // Add default entries to specific configs when needed
    for config in &mut specific {
      if !config.exclude_default_entries {
        config.merge_no_overwrite(&default);
      }
    }

    // Check if some triggers are conflicting with each other
    // For more information, see: https://github.com/espanso/espanso/issues/135
    if default.conflict_check {
      let has_conflicts = Self::has_conflicts(&default, &specific);
      if has_conflicts {
        eprintln!("Warning: some triggers had conflicts and may not behave as intended");
        eprintln!("To turn off this check, add \"conflict_check: false\" in the configuration");
      }
    }

    Ok(LegacyConfigSet { default, specific })
  }

  fn reduce_configs(
    target: LegacyConfig,
    children_map: &HashMap<String, Vec<LegacyConfig>>,
    higher_priority: bool,
  ) -> LegacyConfig {
    if children_map.contains_key(&target.name) {
      let mut target = target;
      for children in children_map.get(&target.name).unwrap() {
        let children = Self::reduce_configs(children.clone(), children_map, higher_priority);
        if higher_priority {
          target.merge_overwrite(children);
        } else {
          target.merge_no_overwrite(&children);
        }
      }
      target
    } else {
      target
    }
  }

  fn has_conflicts(default: &LegacyConfig, specific: &[LegacyConfig]) -> bool {
    let mut sorted_triggers: Vec<String> = default
      .matches
      .iter()
      .flat_map(triggers_for_match)
      .collect();
    sorted_triggers.sort();

    let mut has_conflicts = Self::list_has_conflicts(&sorted_triggers);

    for s in specific {
      let mut specific_triggers: Vec<String> =
        s.matches.iter().flat_map(triggers_for_match).collect();
      specific_triggers.sort();
      has_conflicts |= Self::list_has_conflicts(&specific_triggers);
    }

    has_conflicts
  }

  fn list_has_conflicts(sorted_list: &[String]) -> bool {
    if sorted_list.len() <= 1 {
      return false;
    }

    let mut has_conflicts = false;

    for (i, item) in sorted_list.iter().skip(1).enumerate() {
      let previous = &sorted_list[i];
      if item.starts_with(previous) {
        has_conflicts = true;
        eprintln!(
          "Warning: trigger '{item}' is conflicting with '{previous}' and may not behave as intended"
        );
      }
    }

    has_conflicts
  }
}

// Error handling
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ConfigLoadError {
  FileNotFound,
  UnableToReadFile,
  InvalidYAML(PathBuf, String),
  InvalidConfigDirectory,
  InvalidParameter(PathBuf),
  NameDuplicate(PathBuf),
  UnableToCreateDefaultConfig,
}

impl fmt::Display for ConfigLoadError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
            ConfigLoadError::FileNotFound =>  write!(f, "File not found"),
            ConfigLoadError::UnableToReadFile =>  write!(f, "Unable to read config file"),
            ConfigLoadError::InvalidYAML(path, e) => write!(f, "Error parsing YAML file '{}', invalid syntax: {}", path.to_str().unwrap_or_default(), e),
            ConfigLoadError::InvalidConfigDirectory =>  write!(f, "Invalid config directory"),
            ConfigLoadError::InvalidParameter(path) =>  write!(f, "Invalid parameter in '{}', use of reserved parameters in used defined configs is not permitted", path.to_str().unwrap_or_default()),
            ConfigLoadError::NameDuplicate(path) =>  write!(f, "Found duplicate 'name' in '{}', please use different names", path.to_str().unwrap_or_default()),
            ConfigLoadError::UnableToCreateDefaultConfig =>  write!(f, "Could not generate default config file"),
        }
  }
}

impl Error for ConfigLoadError {
  fn description(&self) -> &str {
    match self {
      ConfigLoadError::FileNotFound => "File not found",
      ConfigLoadError::UnableToReadFile => "Unable to read config file",
      ConfigLoadError::InvalidYAML(_, _) => "Error parsing YAML file, invalid syntax",
      ConfigLoadError::InvalidConfigDirectory => "Invalid config directory",
      ConfigLoadError::InvalidParameter(_) => {
        "Invalid parameter, use of reserved parameters in user defined configs is not permitted"
      }
      ConfigLoadError::NameDuplicate(_) => {
        "Found duplicate 'name' in some configurations, please use different names"
      }
      ConfigLoadError::UnableToCreateDefaultConfig => "Could not generate default config file",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::fs::create_dir_all;
  use std::io::Write;
  use tempfile::{NamedTempFile, TempDir};

  const DEFAULT_CONFIG_FILE_CONTENT: &str = include_str!("res/test/default.yml");
  const TEST_WORKING_CONFIG_FILE: &str = include_str!("res/test/working_config.yml");
  const TEST_CONFIG_FILE_WITH_BAD_YAML: &str = include_str!("res/test/config_with_bad_yaml.yml");

  // Test Configs

  fn create_tmp_file(string: &str) -> NamedTempFile {
    let file = NamedTempFile::new().unwrap();
    file.as_file().write_all(string.as_bytes()).unwrap();
    file
  }

  #[test]
  fn test_config_file_not_found() {
    let config = LegacyConfig::load_config(Path::new("invalid/path"));
    assert!(config.is_err());
    assert_eq!(config.unwrap_err(), ConfigLoadError::FileNotFound);
  }

  #[test]
  fn test_config_file_with_bad_yaml_syntax() {
    let broken_config_file = create_tmp_file(TEST_CONFIG_FILE_WITH_BAD_YAML);
    let config = LegacyConfig::load_config(broken_config_file.path());
    match config {
      Ok(_) => unreachable!(),
      Err(e) => match e {
        ConfigLoadError::InvalidYAML(p, _) => assert_eq!(p, broken_config_file.path().to_owned()),
        _ => unreachable!(),
      },
    }
  }

  #[test]
  fn test_validate_field_macro() {
    let mut result = true;

    validate_field!(result, 3, 3);
    assert!(result);

    validate_field!(result, 10, 3);
    assert!(!result);

    validate_field!(result, 3, 3);
    assert!(!result);
  }

  #[test]
  fn test_user_defined_config_does_not_have_reserved_fields() {
    let working_config_file = create_tmp_file(
      r"

        backend: Clipboard

        ",
    );
    let config = LegacyConfig::load_config(working_config_file.path());
    assert!(config.unwrap().validate_user_defined_config());
  }

  #[test]
  fn test_user_defined_config_has_reserved_fields_config_caching_interval() {
    let working_config_file = create_tmp_file(
      r"

        # This should not happen in an app-specific config
        config_caching_interval: 100

        ",
    );
    let config = LegacyConfig::load_config(working_config_file.path());
    assert!(!config.unwrap().validate_user_defined_config());
  }

  #[test]
  fn test_user_defined_config_has_reserved_fields_toggle_key() {
    let working_config_file = create_tmp_file(
      r"

        # This should not happen in an app-specific config
        toggle_key: CTRL

        ",
    );
    let config = LegacyConfig::load_config(working_config_file.path());
    assert!(!config.unwrap().validate_user_defined_config());
  }

  #[test]
  fn test_user_defined_config_has_reserved_fields_toggle_interval() {
    let working_config_file = create_tmp_file(
      r"

        # This should not happen in an app-specific config
        toggle_interval: 1000

        ",
    );
    let config = LegacyConfig::load_config(working_config_file.path());
    assert!(!config.unwrap().validate_user_defined_config());
  }

  #[test]
  fn test_user_defined_config_has_reserved_fields_backspace_limit() {
    let working_config_file = create_tmp_file(
      r"

        # This should not happen in an app-specific config
        backspace_limit: 10

        ",
    );
    let config = LegacyConfig::load_config(working_config_file.path());
    assert!(!config.unwrap().validate_user_defined_config());
  }

  #[test]
  fn test_config_loaded_correctly() {
    let working_config_file = create_tmp_file(TEST_WORKING_CONFIG_FILE);
    let config = LegacyConfig::load_config(working_config_file.path());
    assert!(config.is_ok());
  }

  // Test ConfigSet

  pub fn create_temp_espanso_directories() -> (TempDir, TempDir) {
    create_temp_espanso_directories_with_default_content(DEFAULT_CONFIG_FILE_CONTENT)
  }

  pub fn create_temp_espanso_directories_with_default_content(
    default_content: &str,
  ) -> (TempDir, TempDir) {
    let data_dir = TempDir::new().expect("unable to create data directory");
    let package_dir = TempDir::new().expect("unable to create package directory");

    let default_path = data_dir.path().join(DEFAULT_CONFIG_FILE_NAME);
    fs::write(default_path, default_content).unwrap();

    (data_dir, package_dir)
  }

  pub fn create_temp_file_in_dir(tmp_dir: &Path, name: &str, content: &str) -> PathBuf {
    let user_defined_path = tmp_dir.join(name);
    let user_defined_path_copy = user_defined_path.clone();
    fs::write(user_defined_path, content).unwrap();

    user_defined_path_copy
  }

  pub fn create_user_config_file(tmp_dir: &Path, name: &str, content: &str) -> PathBuf {
    let user_config_dir = tmp_dir.join(USER_CONFIGS_FOLDER_NAME);
    if !user_config_dir.exists() {
      create_dir_all(&user_config_dir).unwrap();
    }

    create_temp_file_in_dir(&user_config_dir, name, content)
  }

  pub fn create_package_file(
    package_data_dir: &Path,
    package_name: &str,
    filename: &str,
    content: &str,
  ) -> PathBuf {
    let package_dir = package_data_dir.join(package_name);
    if !package_dir.exists() {
      create_dir_all(&package_dir).unwrap();
    }

    create_temp_file_in_dir(&package_dir, filename, content)
  }

  #[test]
  fn test_config_set_default_content_should_work_correctly() {
    let (data_dir, package_dir) = create_temp_espanso_directories();

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path());
    assert!(config_set.is_ok());
  }

  #[test]
  fn test_config_set_load_fail_bad_directory() {
    let config_set = LegacyConfigSet::load(Path::new("invalid/path"), Path::new("invalid/path"));
    assert!(config_set.is_err());
    assert_eq!(
      config_set.unwrap_err(),
      ConfigLoadError::InvalidConfigDirectory
    );
  }

  #[test]
  fn test_config_set_missing_default_file() {
    let data_dir = TempDir::new().expect("unable to create temp directory");
    let package_dir = TempDir::new().expect("unable to create package directory");

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path());
    assert!(config_set.is_err());
    assert_eq!(config_set.unwrap_err(), ConfigLoadError::FileNotFound);
  }

  #[test]
  fn test_config_set_invalid_yaml_syntax() {
    let (data_dir, package_dir) =
      create_temp_espanso_directories_with_default_content(TEST_CONFIG_FILE_WITH_BAD_YAML);
    let default_path = data_dir.path().join(DEFAULT_CONFIG_FILE_NAME);

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path());
    match config_set {
      Ok(_) => unreachable!(),
      Err(e) => match e {
        ConfigLoadError::InvalidYAML(p, _) => assert_eq!(p, default_path),
        _ => unreachable!(),
      },
    }
  }

  #[test]
  fn test_config_set_specific_file_with_reserved_fields() {
    let (data_dir, package_dir) = create_temp_espanso_directories();

    let user_defined_path = create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r"
        config_caching_interval: 10000
        ",
    );
    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path());
    assert!(config_set.is_err());
    assert_eq!(
      config_set.unwrap_err(),
      ConfigLoadError::InvalidParameter(user_defined_path)
    );
  }

  #[test]
  fn test_config_set_specific_file_missing_name_auto_generated() {
    let (data_dir, package_dir) = create_temp_espanso_directories();

    let user_defined_path = create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r"
        backend: Clipboard
        ",
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path());
    assert!(config_set.is_ok());
    assert_eq!(
      config_set.unwrap().specific[0].name,
      user_defined_path.to_str().unwrap_or_default()
    );
  }

  #[test]
  fn test_config_set_specific_file_duplicate_name() {
    let (data_dir, package_dir) = create_temp_espanso_directories();

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r"
        name: specific1
        ",
    );

    create_user_config_file(
      data_dir.path(),
      "specific2.yml",
      r"
        name: specific1
        ",
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path());
    assert!(config_set.is_err());
    assert!(matches!(
      &config_set.unwrap_err(),
      &ConfigLoadError::NameDuplicate(_)
    ));
  }

  #[test]
  fn test_user_defined_config_set_merge_with_parent_matches() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r#"
        matches:
            - trigger: ":lol"
              replace: "LOL"
            - trigger: ":yess"
              replace: "Bob"
        "#,
    );

    create_user_config_file(
      data_dir.path(),
      "specific1.yml",
      r#"
        name: specific1

        matches:
            - trigger: "hello"
              replace: "newstring"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.default.matches.len(), 2);
    assert_eq!(config_set.specific[0].matches.len(), 3);

    assert!(config_set.specific[0]
      .matches
      .iter()
      .any(|x| triggers_for_match(x)[0] == "hello"));
    assert!(config_set.specific[0]
      .matches
      .iter()
      .any(|x| triggers_for_match(x)[0] == ":lol"));
    assert!(config_set.specific[0]
      .matches
      .iter()
      .any(|x| triggers_for_match(x)[0] == ":yess"));
  }

  #[test]
  fn test_user_defined_config_set_merge_with_parent_matches_child_priority() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r#"
        matches:
            - trigger: ":lol"
              replace: "LOL"
            - trigger: ":yess"
              replace: "Bob"
        "#,
    );

    create_user_config_file(
      data_dir.path(),
      "specific2.yml",
      r#"
        name: specific1

        matches:
            - trigger: ":lol"
              replace: "newstring"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.default.matches.len(), 2);
    assert_eq!(config_set.specific[0].matches.len(), 2);

    assert!(config_set.specific[0]
      .matches
      .iter()
      .any(|x| { triggers_for_match(x)[0] == ":lol" && replace_for_match(x) == "newstring" }));
    assert!(config_set.specific[0]
      .matches
      .iter()
      .any(|x| triggers_for_match(x)[0] == ":yess"));
  }

  #[test]
  fn test_user_defined_config_set_exclude_merge_with_parent_matches() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r#"
        matches:
            - trigger: ":lol"
              replace: "LOL"
            - trigger: ":yess"
              replace: "Bob"
        "#,
    );

    create_user_config_file(
      data_dir.path(),
      "specific2.yml",
      r#"
        name: specific1

        exclude_default_entries: true

        matches:
            - trigger: "hello"
              replace: "newstring"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.default.matches.len(), 2);
    assert_eq!(config_set.specific[0].matches.len(), 1);

    assert!(config_set.specific[0]
      .matches
      .iter()
      .any(|x| { triggers_for_match(x)[0] == "hello" && replace_for_match(x) == "newstring" }));
  }

  #[test]
  fn test_only_yaml_files_are_loaded_from_config() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r#"
            matches:
                - trigger: ":lol"
                  replace: "LOL"
                - trigger: ":yess"
                  replace: "Bob"
            "#,
    );

    create_user_config_file(
      data_dir.path(),
      "specific.zzz",
      r#"
        name: specific1

        exclude_default_entries: true

        matches:
            - trigger: "hello"
              replace: "newstring"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 0);
  }

  #[test]
  fn test_hidden_files_are_ignored() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r#"
            matches:
                - trigger: ":lol"
                  replace: "LOL"
                - trigger: ":yess"
                  replace: "Bob"
            "#,
    );

    create_user_config_file(
      data_dir.path(),
      ".specific.yml",
      r#"
        name: specific1

        exclude_default_entries: true

        matches:
            - trigger: "hello"
              replace: "newstring"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 0);
  }

  #[test]
  fn test_config_set_no_parent_configs_works_correctly() {
    let (data_dir, package_dir) = create_temp_espanso_directories();

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r"
        name: specific1
        ",
    );

    create_user_config_file(
      data_dir.path(),
      "specific2.yml",
      r"
        name: specific2
        ",
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 2);
  }

  #[test]
  fn test_config_set_default_parent_works_correctly() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        ",
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
        parent: default

        matches:
            - trigger: "hello"
              replace: "world"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 0);
    assert_eq!(config_set.default.matches.len(), 2);
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "hasta"));
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "hello"));
  }

  #[test]
  fn test_config_set_no_parent_should_not_merge() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        ",
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
        matches:
            - trigger: "hello"
              replace: "world"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 1);
    assert_eq!(config_set.default.matches.len(), 1);
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "hasta"));
    assert!(!config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "hello"));
    assert!(config_set.specific[0]
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "hello"));
  }

  #[test]
  fn test_config_set_default_nested_parent_works_correctly() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        ",
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
        name: custom1
        parent: default

        matches:
            - trigger: "hello"
              replace: "world"
        "#,
    );

    create_user_config_file(
      data_dir.path(),
      "specific2.yml",
      r#"
        parent: custom1

        matches:
            - trigger: "super"
              replace: "mario"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 0);
    assert_eq!(config_set.default.matches.len(), 3);
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "hasta"));
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "hello"));
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "super"));
  }

  #[test]
  fn test_config_set_parent_merge_children_priority_should_be_higher() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        ",
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
        parent: default

        matches:
            - trigger: "hasta"
              replace: "world"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 0);
    assert_eq!(config_set.default.matches.len(), 1);
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| { triggers_for_match(m)[0] == "hasta" && replace_for_match(m) == "world" }));
  }

  #[test]
  fn test_config_set_package_configs_default_merge() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        ",
    );

    create_package_file(
      package_dir.path(),
      "package1",
      "package.yml",
      r#"
        parent: default

        matches:
            - trigger: "harry"
              replace: "potter"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 0);
    assert_eq!(config_set.default.matches.len(), 2);
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "hasta"));
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "harry"));
  }

  #[test]
  fn test_config_set_package_configs_lower_priority_than_user() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        ",
    );

    create_package_file(
      package_dir.path(),
      "package1",
      "package.yml",
      r#"
        parent: default

        matches:
            - trigger: "hasta"
              replace: "potter"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 0);
    assert_eq!(config_set.default.matches.len(), 1);
    assert_eq!(
      triggers_for_match(&config_set.default.matches[0])[0],
      "hasta"
    );
    assert_eq!(
      replace_for_match(&config_set.default.matches[0]),
      "Hasta la vista"
    );
  }

  #[test]
  fn test_config_set_package_configs_without_merge() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        ",
    );

    create_package_file(
      package_dir.path(),
      "package1",
      "package.yml",
      r#"
        matches:
            - trigger: "harry"
              replace: "potter"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 1);
    assert_eq!(config_set.default.matches.len(), 1);
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "hasta"));
    assert!(config_set.specific[0]
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "harry"));
  }

  #[test]
  fn test_config_set_package_configs_multiple_files() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        ",
    );

    create_package_file(
      package_dir.path(),
      "package1",
      "package.yml",
      r#"
        name: package1

        matches:
            - trigger: "harry"
              replace: "potter"
        "#,
    );

    create_package_file(
      package_dir.path(),
      "package1",
      "addon.yml",
      r#"
        parent: package1

        matches:
            - trigger: "ron"
              replace: "weasley"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 1);
    assert_eq!(config_set.default.matches.len(), 1);
    assert!(config_set
      .default
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "hasta"));
    assert!(config_set.specific[0]
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "harry"));
    assert!(config_set.specific[0]
      .matches
      .iter()
      .any(|m| triggers_for_match(m)[0] == "ron"));
  }

  #[test]
  fn test_list_has_conflict_no_conflict() {
    assert!(!LegacyConfigSet::list_has_conflicts(&[
      ":ab".to_owned(),
      ":bc".to_owned()
    ]));
  }

  #[test]
  fn test_list_has_conflict_conflict() {
    let mut list = vec!["ac".to_owned(), "ab".to_owned(), "abc".to_owned()];
    list.sort();
    assert!(LegacyConfigSet::list_has_conflicts(&list));
  }

  #[test]
  fn test_has_conflict_no_conflict() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: ac
              replace: Hasta la vista
            - trigger: bc
              replace: Jon
        ",
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
        name: specific1

        matches:
            - trigger: "hello"
              replace: "world"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert!(!LegacyConfigSet::has_conflicts(
      &config_set.default,
      &config_set.specific
    ),);
  }

  #[test]
  fn test_has_conflict_conflict_in_default() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: ac
              replace: Hasta la vista
            - trigger: bc
              replace: Jon
            - trigger: acb
              replace: Error
        ",
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
        name: specific1

        matches:
            - trigger: "hello"
              replace: "world"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert!(LegacyConfigSet::has_conflicts(
      &config_set.default,
      &config_set.specific
    ),);
  }

  #[test]
  fn test_has_conflict_conflict_in_specific_and_default() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: ac
              replace: Hasta la vista
            - trigger: bc
              replace: Jon
        ",
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
        name: specific1

        matches:
            - trigger: "bcd"
              replace: "Conflict"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert!(LegacyConfigSet::has_conflicts(
      &config_set.default,
      &config_set.specific
    ),);
  }

  #[test]
  fn test_has_conflict_no_conflict_in_specific_and_specific() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r"
        matches:
            - trigger: ac
              replace: Hasta la vista
            - trigger: bc
              replace: Jon
        ",
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
        name: specific1

        matches:
            - trigger: "bad"
              replace: "Conflict"
        "#,
    );
    create_user_config_file(
      data_dir.path(),
      "specific2.yml",
      r#"
        name: specific2

        matches:
            - trigger: "badass"
              replace: "Conflict"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert!(!LegacyConfigSet::has_conflicts(
      &config_set.default,
      &config_set.specific
    ),);
  }

  #[test]
  fn test_config_set_specific_inherits_default_global_vars() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r#"
        global_vars:
            - name: testvar
              type: date
              params:
                format: "%m"
        "#,
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
         global_vars:
            - name: specificvar
              type: date
              params:
                format: "%m"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 1);
    assert_eq!(config_set.default.global_vars.len(), 1);
    assert_eq!(config_set.specific[0].global_vars.len(), 2);
    assert!(config_set.specific[0]
      .global_vars
      .iter()
      .any(|m| name_for_global_var(m) == "testvar"));
    assert!(config_set.specific[0]
      .global_vars
      .iter()
      .any(|m| name_for_global_var(m) == "specificvar"));
  }

  #[test]
  fn test_config_set_default_get_variables_from_specific() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r#"
        global_vars:
            - name: testvar
              type: date
              params:
                format: "%m"
        "#,
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
         parent: default
         global_vars:
            - name: specificvar
              type: date
              params:
                format: "%m"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 0);
    assert_eq!(config_set.default.global_vars.len(), 2);
    assert!(config_set
      .default
      .global_vars
      .iter()
      .any(|m| name_for_global_var(m) == "testvar"));
    assert!(config_set
      .default
      .global_vars
      .iter()
      .any(|m| name_for_global_var(m) == "specificvar"));
  }

  #[test]
  fn test_config_set_specific_dont_inherits_default_global_vars_when_exclude_is_on() {
    let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
      r#"
        global_vars:
            - name: testvar
              type: date
              params:
                format: "%m"
        "#,
    );

    create_user_config_file(
      data_dir.path(),
      "specific.yml",
      r#"
         exclude_default_entries: true

         global_vars:
            - name: specificvar
              type: date
              params:
                format: "%m"
        "#,
    );

    let config_set = LegacyConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
    assert_eq!(config_set.specific.len(), 1);
    assert_eq!(config_set.default.global_vars.len(), 1);
    assert_eq!(config_set.specific[0].global_vars.len(), 1);
    assert!(config_set.specific[0]
      .global_vars
      .iter()
      .any(|m| name_for_global_var(m) == "specificvar"));
  }
}

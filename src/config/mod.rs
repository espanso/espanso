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

extern crate dirs;

use std::path::{Path, PathBuf};
use std::{fs};
use crate::matcher::{Match, MatchVariable};
use std::fs::{File, create_dir_all};
use std::io::Read;
use serde::{Serialize, Deserialize};
use crate::event::KeyModifier;
use crate::keyboard::PasteShortcut;
use std::collections::{HashSet, HashMap};
use log::{error};
use std::fmt;
use std::error::Error;
use walkdir::WalkDir;

pub(crate) mod runtime;

const DEFAULT_CONFIG_FILE_CONTENT : &str = include_str!("../res/config.yml");

pub const DEFAULT_CONFIG_FILE_NAME : &str = "default.yml";
pub const USER_CONFIGS_FOLDER_NAME: &str = "user";

// Default values for primitives
fn default_name() -> String{ "default".to_owned() }
fn default_parent() -> String{ "self".to_owned() }
fn default_filter_title() -> String{ "".to_owned() }
fn default_filter_class() -> String{ "".to_owned() }
fn default_filter_exec() -> String{ "".to_owned() }
fn default_log_level() -> i32 { 0 }
fn default_conflict_check() -> bool{ false }
fn default_ipc_server_port() -> i32 { 34982 }
fn default_use_system_agent() -> bool { true }
fn default_config_caching_interval() -> i32 { 800 }
fn default_word_separators() -> Vec<char> { vec![' ', ',', '.', '?', '!', '\r', '\n', 22u8 as char] }
fn default_toggle_interval() -> u32 { 230 }
fn default_toggle_key() -> KeyModifier { KeyModifier::ALT }
fn default_preserve_clipboard() -> bool {true}
fn default_passive_match_regex() -> String{ "(?P<name>:\\p{L}+)(/(?P<args>.*)/)?".to_owned() }
fn default_passive_arg_delimiter() -> char { '/' }
fn default_passive_arg_escape() -> char { '\\' }
fn default_passive_key() -> KeyModifier { KeyModifier::OFF }
fn default_enable_passive() -> bool { false }
fn default_enable_active() -> bool { true }
fn default_backspace_limit() -> i32 { 3 }
fn default_restore_clipboard_delay() -> i32 { 300 }
fn default_exclude_default_entries() -> bool {false}
fn default_matches() -> Vec<Match> { Vec::new() }
fn default_global_vars() -> Vec<MatchVariable> { Vec::new() }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configs {
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

    #[serde(default = "default_use_system_agent")]
    pub use_system_agent: bool,

    #[serde(default = "default_config_caching_interval")]
    pub config_caching_interval: i32,

    #[serde(default = "default_word_separators")]
    pub word_separators: Vec<char>,  // TODO: add parsing test

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

    #[serde(default = "default_enable_passive")]
    pub enable_passive: bool,

    #[serde(default = "default_enable_active")]
    pub enable_active: bool,

    #[serde(default)]
    pub paste_shortcut: PasteShortcut,

    #[serde(default = "default_backspace_limit")]
    pub backspace_limit: i32,

    #[serde(default = "default_restore_clipboard_delay")]
    pub restore_clipboard_delay: i32,

    #[serde(default)]
    pub backend: BackendType,

    #[serde(default = "default_exclude_default_entries")]
    pub exclude_default_entries: bool,

    #[serde(default = "default_matches")]
    pub matches: Vec<Match>,

    #[serde(default = "default_global_vars")]
    pub global_vars: Vec<MatchVariable>

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

impl Configs {
    /*
     * Validate the Config instance.
     * It makes sure that user defined config instances do not define
     * attributes reserved to the default config.
     */
    fn validate_user_defined_config(&self) -> bool {
        let mut result = true;

        validate_field!(result, self.config_caching_interval, default_config_caching_interval());
        validate_field!(result, self.log_level, default_log_level());
        validate_field!(result, self.conflict_check, default_conflict_check());
        validate_field!(result, self.toggle_key, default_toggle_key());
        validate_field!(result, self.toggle_interval, default_toggle_interval());
        validate_field!(result, self.backspace_limit, default_backspace_limit());
        validate_field!(result, self.ipc_server_port, default_ipc_server_port());
        validate_field!(result, self.use_system_agent, default_use_system_agent());
        validate_field!(result, self.preserve_clipboard, default_preserve_clipboard());
        validate_field!(result, self.passive_match_regex, default_passive_match_regex());
        validate_field!(result, self.passive_arg_delimiter, default_passive_arg_delimiter());
        validate_field!(result, self.passive_arg_escape, default_passive_arg_escape());
        validate_field!(result, self.passive_key, default_passive_key());
        validate_field!(result, self.restore_clipboard_delay, default_restore_clipboard_delay());

        result
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BackendType {
    Inject,
    Clipboard,

    // On Linux systems there is a long standing issue with text injection (which
    // in general is better than Clipboard copy/pasting) that prevents certain
    // apps from correctly handling special characters (such as emojis or accented letters)
    // when injected. For this reason, espanso initially defaulted on the Clipboard
    // backend on Linux, as it was the most reliable (working in 99% of cases),
    // even though it was less efficient and with a few inconveniences (for example, the
    // previous clipboard content being overwritten).
    // The Auto backend tries to take it a step further, by automatically determining
    // when an injection is possible (only ascii characters in the replacement), and falling
    // back to the Clipboard backend otherwise.
    // Should only be used on Linux systems.
    Auto
}
impl Default for BackendType {
    // The default backend varies based on the operating system.
    // On Windows and macOS, the Inject backend is working great and should
    // be preferred as it doesn't override the clipboard.
    // On the other hand, on linux it has many problems due to the bugs
    // of the libxdo used. For this reason, Clipboard will be the default
    // backend on Linux from version v0.3.0

    #[cfg(not(target_os = "linux"))]
    fn default() -> Self {
        BackendType::Inject
    }

    #[cfg(target_os = "linux")]
    fn default() -> Self {
        BackendType::Clipboard
    }
}

impl Configs {
    fn load_config(path: &Path) -> Result<Configs, ConfigLoadError> {
        let file_res = File::open(path);
        if let Ok(mut file) = file_res {
            let mut contents = String::new();
            let res = file.read_to_string(&mut contents);

            if res.is_err() {
                return Err(ConfigLoadError::UnableToReadFile)
            }

            let config_res = serde_yaml::from_str(&contents);

            match config_res {
                Ok(config) => Ok(config),
                Err(e) => {
                    Err(ConfigLoadError::InvalidYAML(path.to_owned(), e.to_string()))
                }
            }
        }else{
            Err(ConfigLoadError::FileNotFound)
        }
    }

    fn merge_config(&mut self, new_config: Configs) {
        // Merge matches
        let mut merged_matches = new_config.matches;
        let mut match_trigger_set = HashSet::new();
        merged_matches.iter().for_each(|m| {
            match_trigger_set.extend(m.triggers.clone());
        });
        let parent_matches : Vec<Match> = self.matches.iter().filter(|&m| {
            !m.triggers.iter().any(|trigger| match_trigger_set.contains(trigger))
        }).cloned().collect();

        merged_matches.extend(parent_matches);
        self.matches = merged_matches;

        // Merge global variables
        let mut merged_global_vars = new_config.global_vars;
        let mut vars_name_set = HashSet::new();
        merged_global_vars.iter().for_each(|m| {
            vars_name_set.insert(m.name.clone());
        });
        let parent_vars : Vec<MatchVariable> = self.global_vars.iter().filter(|&m| {
            !vars_name_set.contains(&m.name)
        }).cloned().collect();

        merged_global_vars.extend(parent_vars);
        self.global_vars = merged_global_vars;
    }

    fn merge_default(&mut self, default: &Configs) {
        // Merge matches
        let mut match_trigger_set = HashSet::new();
        self.matches.iter().for_each(|m| {
            match_trigger_set.extend(m.triggers.clone());
        });
        let default_matches : Vec<Match> = default.matches.iter().filter(|&m| {
            !m.triggers.iter().any(|trigger| match_trigger_set.contains(trigger))
        }).cloned().collect();

        self.matches.extend(default_matches);

        // Merge global variables
        let mut vars_name_set = HashSet::new();
        self.global_vars.iter().for_each(|m| {
            vars_name_set.insert(m.name.clone());
        });
        let default_vars : Vec<MatchVariable> = default.global_vars.iter().filter(|&m| {
            !vars_name_set.contains(&m.name)
        }).cloned().collect();

        self.global_vars.extend(default_vars);

    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigSet {
    pub default: Configs,
    pub specific: Vec<Configs>,
}

impl ConfigSet {
    pub fn load(config_dir: &Path, package_dir: &Path) -> Result<ConfigSet, ConfigLoadError> {
        if !config_dir.is_dir() {
            return Err(ConfigLoadError::InvalidConfigDirectory)
        }

        // Load default configuration
        let default_file = config_dir.join(DEFAULT_CONFIG_FILE_NAME);
        let default = Configs::load_config(default_file.as_path())?;

        // Check that a compatible backend is used, otherwise warn the user
        if cfg!(not(target_os = "linux")) && default.backend == BackendType::Auto {
            eprintln!("Warning: Using Auto backend is only supported on Linux, falling back to Inject backend.");
        }

        // Analyze which config files has to be loaded

        let mut target_files = Vec::new();

        let specific_dir = config_dir.join(USER_CONFIGS_FOLDER_NAME);
        if specific_dir.exists() {
            let dir_entry = WalkDir::new(specific_dir);
            target_files.extend(dir_entry);
        }

        if package_dir.exists() {
            let dir_entry = WalkDir::new(package_dir);
            target_files.extend(dir_entry);
        }

        // Load the user defined config files

        let mut name_set = HashSet::new();
        let mut children_map: HashMap<String, Vec<Configs>> = HashMap::new();
        let mut root_configs = Vec::new();
        root_configs.push(default);

        for entry in target_files {
            if let Ok(entry) = entry {
                let path = entry.path();

                // Skip non-yaml config files
                if path.extension().unwrap_or_default().to_str().unwrap_or_default() != "yml" {
                    continue;
                }

                let mut config = Configs::load_config(&path)?;

                // Make sure the config does not contain reserved fields
                if !config.validate_user_defined_config() {
                    return Err(ConfigLoadError::InvalidParameter(path.to_owned()))
                }

                // No name specified, defaulting to the path name
                if config.name == "default" {
                    config.name = path.to_str().unwrap_or_default().to_owned();
                }

                if name_set.contains(&config.name) {
                    return Err(ConfigLoadError::NameDuplicate(path.to_owned()));
                }

                name_set.insert(config.name.clone());

                if config.parent == "self" {  // No parent, root config
                    root_configs.push(config);
                }else{  // Children config
                    let children_vec = children_map.entry(config.parent.clone()).or_default();
                    children_vec.push(config);
                }
            }else{
                eprintln!("Warning: Unable to read config file: {}", entry.unwrap_err())
            }
        }

        // Merge the children config files
        let mut configs = Vec::new();
        for root_config in root_configs {
            let config = ConfigSet::reduce_configs(root_config, &children_map);
            configs.push(config);
        }

        // Separate default from specific
        let default= configs.get(0).unwrap().clone();
        let mut specific = (&configs[1..]).to_vec().clone();

        // Add default entries to specific configs when needed
        for config in specific.iter_mut() {
            if !config.exclude_default_entries {
                config.merge_default(&default);
            }
        }

        // Check if some triggers are conflicting with each other
        // For more information, see: https://github.com/federico-terzi/espanso/issues/135
        if default.conflict_check {
            for s in specific.iter() {
                let has_conflicts = Self::has_conflicts(&default, &specific);
                if has_conflicts {
                    eprintln!("Warning: some triggers had conflicts and may not behave as intended");
                    eprintln!("To turn off this check, add \"conflict_check: false\" in the configuration");
                }
            }
        }

        Ok(ConfigSet {
            default,
            specific
        })
    }

    fn reduce_configs(target: Configs, children_map: &HashMap<String, Vec<Configs>>) -> Configs {
        if children_map.contains_key(&target.name) {
            let mut target = target;
            for children in children_map.get(&target.name).unwrap() {
                let children = Self::reduce_configs(children.clone(), children_map);
                target.merge_config(children);
            }
            target
        }else{
            target
        }
    }

    pub fn load_default() -> Result<ConfigSet, ConfigLoadError> {
        // Configuration related

        let config_dir = crate::context::get_config_dir();

        let default_file = config_dir.join(DEFAULT_CONFIG_FILE_NAME);

        // If config file does not exist, create one from template
        if !default_file.exists() {
            let result = fs::write(&default_file, DEFAULT_CONFIG_FILE_CONTENT);
            if result.is_err() {
                return Err(ConfigLoadError::UnableToCreateDefaultConfig)
            }
        }

        // Create auxiliary directories

        let user_config_dir = config_dir.join(USER_CONFIGS_FOLDER_NAME);
        if !user_config_dir.exists() {
            let res = create_dir_all(user_config_dir.as_path());
            if res.is_err() {
                return Err(ConfigLoadError::UnableToCreateDefaultConfig)
            }
        }


        // Packages

        let package_dir = crate::context::get_package_dir();
        let res = create_dir_all(package_dir.as_path());
        if res.is_err() {
            return Err(ConfigLoadError::UnableToCreateDefaultConfig)  // TODO: change error type
        }

        return ConfigSet::load(config_dir.as_path(), package_dir.as_path());
    }

    fn has_conflicts(default: &Configs, specific: &Vec<Configs>) -> bool {
        let mut sorted_triggers : Vec<String> = default.matches.iter().flat_map(|t| {
            t.triggers.clone()
        }).collect();
        sorted_triggers.sort();

        let mut has_conflicts = Self::list_has_conflicts(&sorted_triggers);

        for s in specific.iter() {
            let mut specific_triggers : Vec<String> = s.matches.iter().flat_map(|t| {
                t.triggers.clone()
            }).collect();
            specific_triggers.sort();
            has_conflicts |= Self::list_has_conflicts(&specific_triggers);
        }

        has_conflicts
    }

    fn list_has_conflicts(sorted_list: &Vec<String>) -> bool {
        if sorted_list.len() <= 1 {
            return false
        }

        let mut has_conflicts = false;

        for (i, item) in sorted_list.iter().skip(1).enumerate() {
            let previous = &sorted_list[i];
            if item.starts_with(previous) {
                has_conflicts = true;
                eprintln!("Warning: trigger '{}' is conflicting with '{}' and may not behave as intended", item, previous);
            }
        }

        has_conflicts
    }
}

pub trait ConfigManager<'a> {
    fn active_config(&'a self) -> &'a Configs;
    fn default_config(&'a self) -> &'a Configs;
    fn matches(&'a self) -> &'a Vec<Match>;
}

// Error handling
#[derive(Debug, PartialEq)]
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
            ConfigLoadError::InvalidParameter(_) => "Invalid parameter, use of reserved parameters in user defined configs is not permitted",
            ConfigLoadError::NameDuplicate(_) => "Found duplicate 'name' in some configurations, please use different names",
            ConfigLoadError::UnableToCreateDefaultConfig => "Could not generate default config file",
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};
    use std::any::Any;
    use crate::matcher::{TextContent, MatchContentType};

    const TEST_WORKING_CONFIG_FILE : &str = include_str!("../res/test/working_config.yml");
    const TEST_CONFIG_FILE_WITH_BAD_YAML : &str = include_str!("../res/test/config_with_bad_yaml.yml");

    // Test Configs

    fn create_tmp_file(string: &str) -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        file.as_file().write_all(string.as_bytes());
        file
    }

    fn variant_eq<T>(a: &T, b: &T) -> bool {
        std::mem::discriminant(a) == std::mem::discriminant(b)
    }

    #[test]
    fn test_config_file_not_found() {
        let config = Configs::load_config(Path::new("invalid/path"));
        assert_eq!(config.is_err(), true);
        assert_eq!(config.unwrap_err(), ConfigLoadError::FileNotFound);
    }

    #[test]
    fn test_config_file_with_bad_yaml_syntax() {
        let broken_config_file = create_tmp_file(TEST_CONFIG_FILE_WITH_BAD_YAML);
        let config = Configs::load_config(broken_config_file.path());
        match config {
            Ok(_) => {assert!(false)},
            Err(e) => {
                match e {
                    ConfigLoadError::InvalidYAML(p, _) => assert_eq!(p, broken_config_file.path().to_owned()),
                    _ => assert!(false),
                }
                assert!(true);
            },
        }

    }

    #[test]
    fn test_validate_field_macro() {
        let mut result = true;

        validate_field!(result, 3, 3);
        assert_eq!(result, true);

        validate_field!(result, 10, 3);
        assert_eq!(result, false);

        validate_field!(result, 3, 3);
        assert_eq!(result, false);
    }

    #[test]
    fn test_user_defined_config_does_not_have_reserved_fields() {
        let working_config_file = create_tmp_file(r###"

        backend: Clipboard

        "###);
        let config = Configs::load_config(working_config_file.path());
        assert_eq!(config.unwrap().validate_user_defined_config(), true);
    }

    #[test]
    fn test_user_defined_config_has_reserved_fields_config_caching_interval() {
        let working_config_file = create_tmp_file(r###"

        # This should not happen in an app-specific config
        config_caching_interval: 100

        "###);
        let config = Configs::load_config(working_config_file.path());
        assert_eq!(config.unwrap().validate_user_defined_config(), false);
    }

    #[test]
    fn test_user_defined_config_has_reserved_fields_toggle_key() {
        let working_config_file = create_tmp_file(r###"

        # This should not happen in an app-specific config
        toggle_key: CTRL

        "###);
        let config = Configs::load_config(working_config_file.path());
        assert_eq!(config.unwrap().validate_user_defined_config(), false);
    }

    #[test]
    fn test_user_defined_config_has_reserved_fields_toggle_interval() {
        let working_config_file = create_tmp_file(r###"

        # This should not happen in an app-specific config
        toggle_interval: 1000

        "###);
        let config = Configs::load_config(working_config_file.path());
        assert_eq!(config.unwrap().validate_user_defined_config(), false);
    }

    #[test]
    fn test_user_defined_config_has_reserved_fields_backspace_limit() {
        let working_config_file = create_tmp_file(r###"

        # This should not happen in an app-specific config
        backspace_limit: 10

        "###);
        let config = Configs::load_config(working_config_file.path());
        assert_eq!(config.unwrap().validate_user_defined_config(), false);
    }

    #[test]
    fn test_config_loaded_correctly() {
        let working_config_file = create_tmp_file(TEST_WORKING_CONFIG_FILE);
        let config = Configs::load_config(working_config_file.path());
        assert_eq!(config.is_ok(), true);
    }

    // Test ConfigSet

    pub fn create_temp_espanso_directories() -> (TempDir, TempDir) {
        create_temp_espanso_directories_with_default_content(DEFAULT_CONFIG_FILE_CONTENT)
    }

    pub fn create_temp_espanso_directories_with_default_content(default_content: &str) -> (TempDir, TempDir) {
        let data_dir = TempDir::new().expect("unable to create data directory");
        let package_dir = TempDir::new().expect("unable to create package directory");

        let default_path = data_dir.path().join(DEFAULT_CONFIG_FILE_NAME);
        fs::write(default_path, default_content);

        (data_dir, package_dir)
    }

    pub fn create_temp_file_in_dir(tmp_dir: &PathBuf, name: &str, content: &str) -> PathBuf {
        let user_defined_path = tmp_dir.join(name);
        let user_defined_path_copy = user_defined_path.clone();
        fs::write(user_defined_path, content);

        user_defined_path_copy
    }

    pub fn create_user_config_file(tmp_dir: &Path, name: &str, content: &str) -> PathBuf {
        let user_config_dir = tmp_dir.join(USER_CONFIGS_FOLDER_NAME);
        if !user_config_dir.exists() {
            create_dir_all(&user_config_dir);
        }

        create_temp_file_in_dir(&user_config_dir, name, content)
    }

    pub fn create_package_file(package_data_dir: &Path, package_name: &str, filename: &str, content: &str) -> PathBuf {
        let package_dir = package_data_dir.join(package_name);
        if !package_dir.exists() {
            create_dir_all(&package_dir);
        }

        create_temp_file_in_dir(&package_dir, filename, content)
    }

    #[test]
    fn test_config_set_default_content_should_work_correctly() {
        let (data_dir, package_dir) = create_temp_espanso_directories();

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path());
        assert!(config_set.is_ok());
    }

    #[test]
    fn test_config_set_load_fail_bad_directory() {
        let config_set = ConfigSet::load(Path::new("invalid/path"), Path::new("invalid/path"));
        assert_eq!(config_set.is_err(), true);
        assert_eq!(config_set.unwrap_err(), ConfigLoadError::InvalidConfigDirectory);
    }

    #[test]
    fn test_config_set_missing_default_file() {
        let data_dir = TempDir::new().expect("unable to create temp directory");
        let package_dir = TempDir::new().expect("unable to create package directory");

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path());
        assert_eq!(config_set.is_err(), true);
        assert_eq!(config_set.unwrap_err(), ConfigLoadError::FileNotFound);
    }

    #[test]
    fn test_config_set_invalid_yaml_syntax() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
            TEST_CONFIG_FILE_WITH_BAD_YAML
        );
        let default_path = data_dir.path().join(DEFAULT_CONFIG_FILE_NAME);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path());
        match config_set {
            Ok(_) => {assert!(false)},
            Err(e) => {
                match e {
                    ConfigLoadError::InvalidYAML(p, _) => assert_eq!(p, default_path),
                    _ => assert!(false),
                }
                assert!(true);
            },
        }
    }

    #[test]
    fn test_config_set_specific_file_with_reserved_fields() {
        let (data_dir, package_dir) = create_temp_espanso_directories();

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        config_caching_interval: 10000
        "###);
        let user_defined_path_copy = user_defined_path.clone();

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path());
        assert!(config_set.is_err());
        assert_eq!(config_set.unwrap_err(), ConfigLoadError::InvalidParameter(user_defined_path_copy))
    }

    #[test]
    fn test_config_set_specific_file_missing_name_auto_generated() {
        let (data_dir, package_dir) = create_temp_espanso_directories();

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        backend: Clipboard
        "###);
        let user_defined_path_copy = user_defined_path.clone();

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path());
        assert!(config_set.is_ok());
        assert_eq!(config_set.unwrap().specific[0].name, user_defined_path_copy.to_str().unwrap_or_default())
    }

    #[test]
    fn test_config_set_specific_file_duplicate_name() {
        let (data_dir, package_dir) = create_temp_espanso_directories();

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        name: specific1
        "###);

        let user_defined_path2 = create_user_config_file(data_dir.path(), "specific2.yml", r###"
        name: specific1
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path());
        assert!(config_set.is_err());
        assert!(variant_eq(&config_set.unwrap_err(), &ConfigLoadError::NameDuplicate(PathBuf::new())))
    }

    #[test]
    fn test_user_defined_config_set_merge_with_parent_matches() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: ":lol"
              replace: "LOL"
            - trigger: ":yess"
              replace: "Bob"
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific1.yml", r###"
        name: specific1

        matches:
            - trigger: "hello"
              replace: "newstring"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.default.matches.len(), 2);
        assert_eq!(config_set.specific[0].matches.len(), 3);

        assert!(config_set.specific[0].matches.iter().find(|x| x.triggers[0] == "hello").is_some());
        assert!(config_set.specific[0].matches.iter().find(|x| x.triggers[0] == ":lol").is_some());
        assert!(config_set.specific[0].matches.iter().find(|x| x.triggers[0] == ":yess").is_some());
    }

    #[test]
    fn test_user_defined_config_set_merge_with_parent_matches_child_priority() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: ":lol"
              replace: "LOL"
            - trigger: ":yess"
              replace: "Bob"
        "###);

        let user_defined_path2 = create_user_config_file(data_dir.path(), "specific2.yml", r###"
        name: specific1

        matches:
            - trigger: ":lol"
              replace: "newstring"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.default.matches.len(), 2);
        assert_eq!(config_set.specific[0].matches.len(), 2);

        assert!(config_set.specific[0].matches.iter().find(|x| {
            if let MatchContentType::Text(content) = &x.content {
                x.triggers[0] == ":lol" && content.replace == "newstring"
            }else{
                false
            }
        }).is_some());
        assert!(config_set.specific[0].matches.iter().find(|x| x.triggers[0] == ":yess").is_some());
    }

    #[test]
    fn test_user_defined_config_set_exclude_merge_with_parent_matches() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: ":lol"
              replace: "LOL"
            - trigger: ":yess"
              replace: "Bob"
        "###);

        let user_defined_path2 = create_user_config_file(data_dir.path(), "specific2.yml", r###"
        name: specific1

        exclude_default_entries: true

        matches:
            - trigger: "hello"
              replace: "newstring"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.default.matches.len(), 2);
        assert_eq!(config_set.specific[0].matches.len(), 1);

        assert!(config_set.specific[0].matches.iter().find(|x| {
            if let MatchContentType::Text(content) = &x.content {
                x.triggers[0] == "hello" && content.replace == "newstring"
            }else{
                false
            }
        }).is_some());
    }

    #[test]
    fn test_only_yaml_files_are_loaded_from_config() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(
            r###"
            matches:
                - trigger: ":lol"
                  replace: "LOL"
                - trigger: ":yess"
                  replace: "Bob"
            "###
        );

        let user_defined_path2 = create_user_config_file(data_dir.path(), "specific.zzz", r###"
        name: specific1

        exclude_default_entries: true

        matches:
            - trigger: "hello"
              replace: "newstring"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 0);
    }

    #[test]
    fn test_config_set_no_parent_configs_works_correctly() {
        let (data_dir, package_dir) = create_temp_espanso_directories();

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        name: specific1
        "###);

        let user_defined_path2 = create_user_config_file(data_dir.path(), "specific2.yml", r###"
        name: specific2
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 2);
    }

    #[test]
    fn test_config_set_default_parent_works_correctly() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        parent: default

        matches:
            - trigger: "hello"
              replace: "world"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 0);
        assert_eq!(config_set.default.matches.len(), 2);
        assert!(config_set.default.matches.iter().any(|m| m.triggers[0] == "hasta"));
        assert!(config_set.default.matches.iter().any(|m| m.triggers[0] == "hello"));
    }

    #[test]
    fn test_config_set_no_parent_should_not_merge() {
        let (data_dir, package_dir)= create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        matches:
            - trigger: "hello"
              replace: "world"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 1);
        assert_eq!(config_set.default.matches.len(), 1);
        assert!(config_set.default.matches.iter().any(|m| m.triggers[0] == "hasta"));
        assert!(!config_set.default.matches.iter().any(|m| m.triggers[0] == "hello"));
        assert!(config_set.specific[0].matches.iter().any(|m| m.triggers[0] == "hello"));
    }

    #[test]
    fn test_config_set_default_nested_parent_works_correctly() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        name: custom1
        parent: default

        matches:
            - trigger: "hello"
              replace: "world"
        "###);

        let user_defined_path2 = create_user_config_file(data_dir.path(), "specific2.yml", r###"
        parent: custom1

        matches:
            - trigger: "super"
              replace: "mario"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 0);
        assert_eq!(config_set.default.matches.len(), 3);
        assert!(config_set.default.matches.iter().any(|m| m.triggers[0] == "hasta"));
        assert!(config_set.default.matches.iter().any(|m| m.triggers[0] == "hello"));
        assert!(config_set.default.matches.iter().any(|m| m.triggers[0] == "super"));
    }

    #[test]
    fn test_config_set_parent_merge_children_priority_should_be_higher() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        parent: default

        matches:
            - trigger: "hasta"
              replace: "world"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 0);
        assert_eq!(config_set.default.matches.len(), 1);
        assert!(config_set.default.matches.iter().any(|m| {
            if let MatchContentType::Text(content) = &m.content {
                m.triggers[0] == "hasta" && content.replace == "world"
            }else{
                false
            }
        }));
    }

    #[test]
    fn test_config_set_package_configs_default_merge() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        "###);

        let package_path = create_package_file(package_dir.path(), "package1", "package.yml", r###"
        parent: default

        matches:
            - trigger: "harry"
              replace: "potter"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 0);
        assert_eq!(config_set.default.matches.len(), 2);
        assert!(config_set.default.matches.iter().any(|m| m.triggers[0] == "hasta"));
        assert!(config_set.default.matches.iter().any(|m| m.triggers[0] == "harry"));
    }

    #[test]
    fn test_config_set_package_configs_without_merge() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        "###);

        let package_path = create_package_file(package_dir.path(), "package1", "package.yml", r###"
        matches:
            - trigger: "harry"
              replace: "potter"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 1);
        assert_eq!(config_set.default.matches.len(), 1);
        assert!(config_set.default.matches.iter().any(|m| m.triggers[0] == "hasta"));
        assert!(config_set.specific[0].matches.iter().any(|m| m.triggers[0] == "harry"));
    }

    #[test]
    fn test_config_set_package_configs_multiple_files() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: hasta
              replace: Hasta la vista
        "###);

        let package_path = create_package_file(package_dir.path(), "package1", "package.yml", r###"
        name: package1

        matches:
            - trigger: "harry"
              replace: "potter"
        "###);

        let package_path2 = create_package_file(package_dir.path(), "package1", "addon.yml", r###"
        parent: package1

        matches:
            - trigger: "ron"
              replace: "weasley"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 1);
        assert_eq!(config_set.default.matches.len(), 1);
        assert!(config_set.default.matches.iter().any(|m| m.triggers[0] == "hasta"));
        assert!(config_set.specific[0].matches.iter().any(|m| m.triggers[0] == "harry"));
        assert!(config_set.specific[0].matches.iter().any(|m| m.triggers[0] == "ron"));
    }

    #[test]
    fn test_list_has_conflict_no_conflict() {
        assert_eq!(ConfigSet::list_has_conflicts(&vec!(":ab".to_owned(), ":bc".to_owned())), false);
    }

    #[test]
    fn test_list_has_conflict_conflict() {
        let mut list = vec!("ac".to_owned(), "ab".to_owned(), "abc".to_owned());
        list.sort();
        assert_eq!(ConfigSet::list_has_conflicts(&list), true);
    }

    #[test]
    fn test_has_conflict_no_conflict() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: ac
              replace: Hasta la vista
            - trigger: bc
              replace: Jon
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        name: specific1

        matches:
            - trigger: "hello"
              replace: "world"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(ConfigSet::has_conflicts(&config_set.default, &config_set.specific), false);
    }

    #[test]
    fn test_has_conflict_conflict_in_default() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: ac
              replace: Hasta la vista
            - trigger: bc
              replace: Jon
            - trigger: acb
              replace: Error
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        name: specific1

        matches:
            - trigger: "hello"
              replace: "world"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(ConfigSet::has_conflicts(&config_set.default, &config_set.specific), true);
    }

    #[test]
    fn test_has_conflict_conflict_in_specific_and_default() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: ac
              replace: Hasta la vista
            - trigger: bc
              replace: Jon
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        name: specific1

        matches:
            - trigger: "bcd"
              replace: "Conflict"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(ConfigSet::has_conflicts(&config_set.default, &config_set.specific), true);
    }

    #[test]
    fn test_has_conflict_no_conflict_in_specific_and_specific() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        matches:
            - trigger: ac
              replace: Hasta la vista
            - trigger: bc
              replace: Jon
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
        name: specific1

        matches:
            - trigger: "bad"
              replace: "Conflict"
        "###);
        let user_defined_path2 = create_user_config_file(data_dir.path(), "specific2.yml", r###"
        name: specific2

        matches:
            - trigger: "badass"
              replace: "Conflict"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(ConfigSet::has_conflicts(&config_set.default, &config_set.specific), false);
    }

    #[test]
    fn test_config_set_specific_inherits_default_global_vars() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        global_vars:
            - name: testvar
              type: date
              params:
                format: "%m"
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
         global_vars:
            - name: specificvar
              type: date
              params:
                format: "%m"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 1);
        assert_eq!(config_set.default.global_vars.len(), 1);
        assert_eq!(config_set.specific[0].global_vars.len(), 2);
        assert!(config_set.specific[0].global_vars.iter().any(|m| m.name == "testvar"));
        assert!(config_set.specific[0].global_vars.iter().any(|m| m.name == "specificvar"));
    }

    #[test]
    fn test_config_set_default_get_variables_from_specific() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        global_vars:
            - name: testvar
              type: date
              params:
                format: "%m"
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
         parent: default
         global_vars:
            - name: specificvar
              type: date
              params:
                format: "%m"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 0);
        assert_eq!(config_set.default.global_vars.len(), 2);
        assert!(config_set.default.global_vars.iter().any(|m| m.name == "testvar"));
        assert!(config_set.default.global_vars.iter().any(|m| m.name == "specificvar"));
    }

    #[test]
    fn test_config_set_specific_dont_inherits_default_global_vars_when_exclude_is_on() {
        let (data_dir, package_dir) = create_temp_espanso_directories_with_default_content(r###"
        global_vars:
            - name: testvar
              type: date
              params:
                format: "%m"
        "###);

        let user_defined_path = create_user_config_file(data_dir.path(), "specific.yml", r###"
         exclude_default_entries: true

         global_vars:
            - name: specificvar
              type: date
              params:
                format: "%m"
        "###);

        let config_set = ConfigSet::load(data_dir.path(), package_dir.path()).unwrap();
        assert_eq!(config_set.specific.len(), 1);
        assert_eq!(config_set.default.global_vars.len(), 1);
        assert_eq!(config_set.specific[0].global_vars.len(), 1);
        assert!(config_set.specific[0].global_vars.iter().any(|m| m.name == "specificvar"));
    }
}
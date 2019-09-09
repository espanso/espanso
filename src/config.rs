extern crate dirs;

use std::path::Path;
use std::fs;
use crate::matcher::Match;
use std::fs::{File, create_dir_all};
use std::io::Read;
use serde::{Serialize, Deserialize};
use crate::keyboard::KeyModifier;
use crate::system::SystemManager;
use std::collections::HashSet;
use regex::Regex;
use std::process::exit;
use log::{debug, info, warn, error};
use std::cell::RefCell;
use std::time::SystemTime;

// TODO: add documentation link
const DEFAULT_CONFIG_FILE_CONTENT : &str = include_str!("res/config.yaml");

const DEFAULT_CONFIG_FILE_NAME : &str = "default.yaml";

// Default values for primitives
fn default_name() -> String{ "default".to_owned() }
fn default_filter_title() -> String{ "".to_owned() }
fn default_filter_class() -> String{ "".to_owned() }
fn default_filter_exec() -> String{ "".to_owned() }
fn default_disable() -> bool{ false }
fn default_toggle_interval() -> u32 { 230 }
fn default_backspace_limit() -> i32 { 3 }
fn default_matches() -> Vec<Match> { Vec::new() }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configs {
    #[serde(default = "default_name")]
    pub name: String,

    #[serde(default = "default_filter_title")]
    pub filter_title: String,

    #[serde(default = "default_filter_class")]
    pub filter_class: String,

    #[serde(default = "default_filter_exec")]
    pub filter_exec: String,

    #[serde(default = "default_disable")]
    pub disabled: bool,

    #[serde(default)]
    pub toggle_key: KeyModifier,

    #[serde(default = "default_toggle_interval")]
    pub toggle_interval: u32,

    #[serde(default = "default_backspace_limit")]
    pub backspace_limit: i32,

    #[serde(default)]
    pub backend: BackendType,

    #[serde(default = "default_matches")]
    pub matches: Vec<Match>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BackendType {
    Inject,
    Clipboard
}
impl Default for BackendType {
    fn default() -> Self {
        BackendType::Inject
    }
}

impl Configs {
    fn load_config(path: &Path) -> Configs {
        let file_res = File::open(path);
        if let Ok(mut file) = file_res {
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .expect("Unable to read config file");
            let config: Configs = serde_yaml::from_str(&contents)
                .expect("Unable to parse config file, invalid YAML syntax");

            config
        }else{
            panic!("Config file not found...")
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigSet {
    default: Configs,
    specific: Vec<Configs>,
}

impl ConfigSet { // TODO: tests
    pub fn load(dir_path: &Path) -> ConfigSet {
        if !dir_path.is_dir() {
            error!("Invalid config directory");
            exit(2);
        }

        let default_file = dir_path.join(DEFAULT_CONFIG_FILE_NAME);
        let default = Configs::load_config(default_file.as_path());

        let mut specific = Vec::new();

        // Used to make sure no duplicates are present
        let mut name_set = HashSet::new();

        for entry in fs::read_dir(dir_path)
            .expect("Cannot read espanso config directory!") {

            let entry = entry;
            if let Ok(entry) = entry {
                let path = entry.path();

                // Skip the default one, already loaded
                if path.file_name().unwrap_or("".as_ref()) == "default.yaml" {
                    continue;
                }

                let config = Configs::load_config(path.as_path());

                if config.name == "default" {
                    error!("Error while parsing {}, please specify a 'name' field", path.to_str().unwrap_or(""));
                    continue;
                }

                if name_set.contains(&config.name) {
                    error!("Error while parsing {} : the specified name is already used, please use another one",  path.to_str().unwrap_or(""));
                    continue;
                }

                name_set.insert(config.name.clone());
                specific.push(config);
            }
        }

        ConfigSet {
            default,
            specific
        }
    }

    pub fn load_default() -> ConfigSet {
        let res = dirs::home_dir();
        if let Some(home_dir) = res {
            let espanso_dir = home_dir.join(".espanso");

            // Create the espanso dir if id doesn't exist
            let res = create_dir_all(espanso_dir.as_path());

            if let Ok(_) = res {
                let default_file = espanso_dir.join(DEFAULT_CONFIG_FILE_NAME);

                // If config file does not exist, create one from template
                if !default_file.exists() {
                    fs::write(&default_file, DEFAULT_CONFIG_FILE_CONTENT)
                        .expect("Unable to write default config file");
                }

                return ConfigSet::load(espanso_dir.as_path())
            }
        }

        error!("Could not generate default position for config file");
        exit(1);
    }
}

pub trait ConfigManager<'a> {
    fn active_config(&'a self) -> &'a Configs;
    fn default_config(&'a self) -> &'a Configs;
    fn matches(&'a self) -> &'a Vec<Match>;
}

pub struct RuntimeConfigManager<'a, S: SystemManager> {
    set: ConfigSet,
    title_regexps: Vec<Option<Regex>>,
    class_regexps: Vec<Option<Regex>>,
    exec_regexps: Vec<Option<Regex>>,

    system_manager: S,

    // Cache
    last_config_update: RefCell<SystemTime>,
    last_config: RefCell<Option<&'a Configs>>
}

impl <'a, S: SystemManager> RuntimeConfigManager<'a, S> {
    pub fn new<'b>(set: ConfigSet, system_manager: S) -> RuntimeConfigManager<'b, S> {
        // Compile all the regexps
        let title_regexps = set.specific.iter().map(
            |config| {
                if config.filter_title.is_empty() {
                    None
                }else{
                    let res = Regex::new(&config.filter_title);
                    if let Ok(regex) = res {
                        Some(regex)
                    }else{
                        warn!("Invalid regex in 'filter_title' field of configuration {}, ignoring it...", config.name);
                        None
                    }
                }
            }
        ).collect();

        let class_regexps = set.specific.iter().map(
            |config| {
                if config.filter_class.is_empty() {
                    None
                }else{
                    let res = Regex::new(&config.filter_class);
                    if let Ok(regex) = res {
                        Some(regex)
                    }else{
                        warn!("Invalid regex in 'filter_class' field of configuration {}, ignoring it...", config.name);
                        None
                    }
                }
            }
        ).collect();

        let exec_regexps = set.specific.iter().map(
            |config| {
                if config.filter_exec.is_empty() {
                    None
                }else{
                    let res = Regex::new(&config.filter_exec);
                    if let Ok(regex) = res {
                        Some(regex)
                    }else{
                        warn!("Invalid regex in 'filter_exec' field of configuration {}, ignoring it...", config.name);
                        None
                    }
                }
            }
        ).collect();

        let last_config_update = RefCell::new(SystemTime::now());
        let last_config = RefCell::new(None);

        RuntimeConfigManager {
            set,
            title_regexps,
            class_regexps,
            exec_regexps,
            system_manager,
            last_config_update,
            last_config
        }
    }

    fn calculate_active_config(&'a self) -> &'a Configs {
        // TODO: optimize performance by avoiding some of these checks if no Configs use the filters

        debug!("Requested config for window:");

        let active_title = self.system_manager.get_current_window_title();

        if let Some(title) = active_title {
            debug!("=> Title: '{}'", title);

            for (i, regex) in self.title_regexps.iter().enumerate() {
                if let Some(regex) = regex {
                    if regex.is_match(&title) {
                        debug!("Matched 'filter_title' for '{}' config, using custom settings.",
                               self.set.specific[i].name);

                        return &self.set.specific[i]
                    }
                }
            }
        }

        let active_executable = self.system_manager.get_current_window_executable();

        if let Some(executable) = active_executable {
            debug!("=> Executable: '{}'", executable);

            for (i, regex) in self.exec_regexps.iter().enumerate() {
                if let Some(regex) = regex {
                    if regex.is_match(&executable) {
                        debug!("Matched 'filter_exec' for '{}' config, using custom settings.",
                               self.set.specific[i].name);

                        return &self.set.specific[i]
                    }
                }
            }
        }

        let active_class = self.system_manager.get_current_window_class();

        if let Some(class) = active_class {
            debug!("=> Class: '{}'", class);

            for (i, regex) in self.class_regexps.iter().enumerate() {
                if let Some(regex) = regex {
                    if regex.is_match(&class) {
                        debug!("Matched 'filter_class' for '{}' config, using custom settings.",
                               self.set.specific[i].name);

                        return &self.set.specific[i]
                    }
                }
            }
        }

        // No matches, return the default mapping
        debug!("No matches for custom configs, using default settings.");
        &self.set.default
    }
}

impl <'a, S: SystemManager> ConfigManager<'a> for RuntimeConfigManager<'a, S> {
    fn active_config(&'a self) -> &'a Configs {
        let mut last_config_update = self.last_config_update.borrow_mut();
        if let Ok(elapsed) = (*last_config_update).elapsed() {
            *last_config_update = SystemTime::now();

            if elapsed.as_millis() < 800 {  // TODO: make config option
                let last_config = self.last_config.borrow();
                if let Some(cached_config) = *last_config {
                    debug!("Using cached config");
                    return cached_config;
                }
            }
        }

        let config = self.calculate_active_config();

        let mut last_config = self.last_config.borrow_mut();
        *last_config = Some(config);

        config
    }

    fn default_config(&'a self) -> &'a Configs {
        &self.set.default
    }

    fn matches(&'a self) -> &'a Vec<Match> {
        &self.active_config().matches
    }
}
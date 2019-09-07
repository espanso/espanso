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

// TODO: add documentation link
const DEFAULT_CONFIG_FILE_CONTENT : &str = include_str!("res/config.yaml");

const DEFAULT_CONFIG_FILE_NAME : &str = "default.yaml";


// Default values for primitives
fn default_name() -> String{ "default".to_owned() }
fn default_filter_title() -> String{ "".to_owned() }
fn default_toggle_interval() -> u32 { 230 }
fn default_backspace_limit() -> i32 { 3 }
fn default_matches() -> Vec<Match> { Vec::new() }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configs {
    #[serde(default = "default_name")]
    pub name: String,

    #[serde(default = "default_filter_title")]
    pub filter_title: String,

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
            panic!("Invalid config directory");
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
                    panic!(format!("Error while parsing {} : please a name", path.to_str().unwrap()))
                }

                if name_set.contains(&config.name) {
                    panic!(format!("Error while parsing {} : the specified name is already used, please specify another one",  path.to_str().unwrap()))
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

        panic!("Could not generate default position for config file");
    }
}

pub trait ConfigManager {
    fn toggle_key(&self) -> &KeyModifier;
    fn toggle_interval(&self) -> u32;
    fn backspace_limit(&self) -> i32;
    fn backend(&self) -> &BackendType;
    fn matches(&self) -> &Vec<Match>;
}

pub struct RuntimeConfigManager<S: SystemManager> {
    set: ConfigSet,
    title_regexps: Vec<Option<Regex>>,

    system_manager: S
}

impl <S: SystemManager> RuntimeConfigManager<S> {
    pub fn new(set: ConfigSet, system_manager: S) -> RuntimeConfigManager<S> {
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
                        // TODO: log invalid regex error
                        None
                    }
                }
            }
        ).collect();

        RuntimeConfigManager {
            set,
            title_regexps,
            system_manager
        }
    }
}

impl <S: SystemManager> RuntimeConfigManager<S> {
    fn active_config(&self) -> &Configs {
        // TODO: optimize performance by avoiding some of these checks if no Configs use the filters

        let active_title = self.system_manager.get_current_window_title();

        if let Some(title) = active_title {
            for (i, regex) in self.title_regexps.iter().enumerate() {
                if let Some(regex) = regex {
                    if regex.is_match(&title) {
                        return &self.set.specific[i]
                    }
                }
            }
        }

        // No matches, return the default mapping
        &self.set.default
    }
}

impl <S: SystemManager> ConfigManager for RuntimeConfigManager<S> {
    fn toggle_key(&self) -> &KeyModifier {
        &self.active_config().toggle_key
    }

    fn toggle_interval(&self) -> u32 {
        self.active_config().toggle_interval
    }

    fn backspace_limit(&self) -> i32 {
        self.active_config().backspace_limit
    }

    fn backend(&self) -> &BackendType {
        &self.active_config().backend
    }

    fn matches(&self) -> &Vec<Match> {
        &self.active_config().matches
    }
}
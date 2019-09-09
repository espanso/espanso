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
use std::process::exit;
use log::{debug, info, warn, error};

pub(crate) mod runtime;

// TODO: add documentation link
const DEFAULT_CONFIG_FILE_CONTENT : &str = include_str!("../res/config.yaml");

const DEFAULT_CONFIG_FILE_NAME : &str = "default.yaml";

// Default values for primitives
fn default_name() -> String{ "default".to_owned() }
fn default_filter_title() -> String{ "".to_owned() }
fn default_filter_class() -> String{ "".to_owned() }
fn default_filter_exec() -> String{ "".to_owned() }
fn default_disable() -> bool{ false }
fn default_config_caching_interval() -> i32 { 800 }
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

    #[serde(default = "default_config_caching_interval")]
    pub config_caching_interval: i32,

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
extern crate dirs;

use std::path::Path;
use std::fs;
use crate::matcher::Match;
use std::fs::{File, create_dir_all};
use std::io::Read;
use serde::{Serialize, Deserialize};
use crate::keyboard::KeyModifier;

// TODO: add documentation link
const DEFAULT_CONFIG_FILE_CONTENT : &str = include_str!("res/config.yaml");

const DEFAULT_CONFIG_FILE_NAME : &str = "default.yaml";


// Default values for primitives
fn default_toggle_interval() -> u32 {
    230
}
fn default_backspace_limit() -> i32 {
    3
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configs {
    #[serde(default)]
    pub toggle_key: KeyModifier,

    #[serde(default = "default_toggle_interval")]
    pub toggle_interval: u32,

    #[serde(default = "default_backspace_limit")]
    pub backspace_limit: i32,

    #[serde(default)]
    pub backend: BackendType,

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

impl ConfigSet {
    pub fn load(dir_path: &Path) -> ConfigSet {
        if !dir_path.is_dir() {
            panic!("Invalid config directory");
        }

        let default_file = dir_path.join(DEFAULT_CONFIG_FILE_NAME);
        let default = Configs::load_config(default_file.as_path());

        let mut specific = Vec::new();

        for entry in fs::read_dir(dir_path)
            .expect("Cannot read espanso config directory!") {

            let entry = entry;
            if let Ok(entry) = entry {
                let path = entry.path();

                let config = Configs::load_config(path.as_path());
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

    pub fn toggle_key(&self) -> &KeyModifier {
        &self.default.toggle_key
    }

    pub fn toggle_interval(&self) -> u32 {
        self.default.toggle_interval
    }

    pub fn backspace_limit(&self) -> i32 {
        self.default.backspace_limit
    }

    pub fn backend(&self) -> &BackendType {
        &BackendType::Inject // TODO make dynamic based on system current active app
    }

    pub fn matches(&self) -> &Vec<Match> {
        &self.default.matches
    }
}
extern crate dirs;

use std::path::Path;
use std::fs;
use crate::matcher::Match;
use std::fs::File;
use std::io::Read;
use serde::{Serialize, Deserialize};
use crate::keyboard::KeyModifier;
use crate::keyboard::KeyModifier::*;

// TODO: add documentation link
const DEFAULT_CONFIG_FILE_CONTENT : &str = include_str!("res/config.yaml");

// Default values for primitives

fn default_toggle_interval() -> u32 {
    230
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configs {
    #[serde(default)]
    pub toggle_key: KeyModifier,

    #[serde(default = "default_toggle_interval")]
    pub toggle_interval: u32,

    pub matches: Vec<Match>
}

impl Configs {
    pub fn load(path: &Path) -> Configs {
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

    pub fn load_default() -> Configs {
        let res = dirs::home_dir();
        if let Some(home_dir) = res {
            let default_file = home_dir.join(".espanso");

            // If config file does not exist, create one from template
            if !default_file.exists() {
                fs::write(&default_file, DEFAULT_CONFIG_FILE_CONTENT)
                    .expect("Unable to write default config file");
            }

            Configs::load(default_file.as_path())
        }else{
            panic!("Could not generate default position for config file");
        }
    }
}
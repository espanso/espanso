use std::path::Path;
use crate::matcher::Match;
use std::fs::File;
use std::io::Read;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Configs {
    pub matches: Vec<Match>
}

impl Configs {
    pub fn load(path: &Path) -> Configs {
        let mut file_res = File::open(path);
        if let Ok(mut file) = file_res {
            let mut contents = String::new();
            file.read_to_string(&mut contents);
            let config: Configs = serde_yaml::from_str(&contents).unwrap();

            config
        }else{
            panic!("Config file not found...")
        }
    }


}
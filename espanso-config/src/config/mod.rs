use std::collections::HashSet;
use anyhow::Result;


mod yaml;
mod path;

pub struct Config {
  pub label: Option<String>,
  //pub backend: 
  pub match_files: Vec<String>,
}

impl Config {
}

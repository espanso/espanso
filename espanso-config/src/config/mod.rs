use std::collections::HashSet;
use anyhow::Result;


mod yaml;
mod path;
mod util;

pub struct Config {
  pub label: Option<String>,
  //pub backend: 
  pub match_paths: HashSet<String>,
}

impl Config {
}

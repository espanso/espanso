use std::collections::HashSet;

mod path;
mod parse;
mod util;
mod resolve;
mod store;

pub trait Config {
  fn label(&self) -> &str;
  fn match_paths(&self) -> &HashSet<String>;

  fn is_match(&self, app: &AppProperties) -> bool;
}

pub trait ConfigStore<'a> {
  fn default(&'a self) -> &'a dyn Config;
  fn active(&'a self, app: &AppProperties) -> &'a dyn Config;
}

pub struct AppProperties<'a> {
  pub title: Option<&'a str>,
  pub class: Option<&'a str>,
  pub exec: Option<&'a str>,
}
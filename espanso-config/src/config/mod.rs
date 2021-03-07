use std::collections::HashSet;

mod path;
mod parse;
mod util;
mod resolve;

pub trait Config {
  fn label(&self) -> &str;
  fn match_paths(&self) -> &HashSet<String>;
}
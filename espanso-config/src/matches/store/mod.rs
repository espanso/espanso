use super::{Match, Variable};

mod default;

pub trait MatchStore {
  fn load(&mut self, paths: &[String]);
  fn query(&self, paths: &[String]) -> MatchSet;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchSet<'a> {
  pub matches: Vec<&'a Match>,
  pub global_vars: Vec<&'a Variable>,
}

pub fn new() -> impl MatchStore {
  // TODO: here we can replace the DefaultMatchStore with a caching wrapper
  // that returns the same response for the given "paths" query
  default::DefaultMatchStore::new()
}
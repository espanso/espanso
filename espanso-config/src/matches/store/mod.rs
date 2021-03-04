use super::{Match, Variable};

mod default;

pub trait MatchStore {
  fn load(&mut self, paths: &[String]);
  fn query_set(&self, paths: &[String]) -> MatchSet;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchSet<'a> {
  pub matches: Vec<&'a Match>,
  pub global_vars: Vec<&'a Variable>,
}

pub fn new() -> impl MatchStore {
  default::DefaultMatchStore::new()
}
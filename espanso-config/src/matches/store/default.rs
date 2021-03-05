use log::error;
use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
};

use super::{MatchSet, MatchStore};
use crate::{
  counter::StructId,
  matches::{group::MatchGroup, Match, Variable},
};

// TODO: implement store according to notes
pub(crate) struct DefaultMatchStore {
  pub groups: HashMap<String, MatchGroup>,
}

impl DefaultMatchStore {
  pub fn new() -> Self {
    Self {
      groups: HashMap::new(),
    }
  }
}

impl MatchStore for DefaultMatchStore {
  // TODO: test
  // TODO: test cyclical imports
  fn load(&mut self, paths: &[String]) {
    // Because match groups can imports other match groups,
    // we have to load them recursively starting from the
    // top-level ones.
    load_match_groups_recursively(&mut self.groups, paths);
  }

  // TODO: test
  // TODO: test for cyclical imports
  fn query_set(&self, paths: &[String]) -> MatchSet {
    let mut matches: Vec<&Match> = Vec::new();
    let mut global_vars: Vec<&Variable> = Vec::new();
    let mut visited_paths = HashSet::new();
    let mut visited_matches = HashSet::new();
    let mut visited_global_vars = HashSet::new();

    query_matches_for_paths(
      &self.groups,
      &mut visited_paths,
      &mut visited_matches,
      &mut visited_global_vars,
      &mut matches,
      &mut global_vars,
      paths,
    );

    MatchSet {
      matches,
      global_vars,
    }
  }
}

fn load_match_groups_recursively(groups: &mut HashMap<String, MatchGroup>, paths: &[String]) {
  for path in paths.iter() {
    if !groups.contains_key(path) {
      let group_path = PathBuf::from(path);
      match MatchGroup::load(&group_path) {
        Ok(group) => {
          load_match_groups_recursively(groups, &group.imports);
          groups.insert(path.clone(), group);
        }
        Err(error) => {
          error!("unable to load match group: {:?}", error);
        }
      }
    }
  }
}

// TODO: test
fn query_matches_for_paths<'a>(
  groups: &'a HashMap<String, MatchGroup>,
  visited_paths: &mut HashSet<String>,
  visited_matches: &mut HashSet<StructId>,
  visited_global_vars: &mut HashSet<StructId>,
  matches: &mut Vec<&'a Match>,
  global_vars: &mut Vec<&'a Variable>,
  paths: &[String],
) {
  for path in paths.iter() {
    if !visited_paths.contains(path) {
      if let Some(group) = groups.get(path) {
        for m in group.matches.iter() {
          if !visited_matches.contains(&m._id) {
            matches.push(m);
            visited_matches.insert(m._id);
          }
        }

        for var in group.global_vars.iter() {
          if !visited_global_vars.contains(&var._id) {
            global_vars.push(var);
            visited_global_vars.insert(var._id);
          }
        }

        query_matches_for_paths(
          groups,
          visited_paths,
          visited_matches,
          visited_global_vars,
          matches,
          global_vars,
          &group.imports,
        )
      }

      visited_paths.insert(path.clone());
    }
  }
}

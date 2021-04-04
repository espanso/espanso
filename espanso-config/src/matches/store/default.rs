/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

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

pub(crate) struct DefaultMatchStore {
  pub groups: HashMap<String, MatchGroup>,
}

impl DefaultMatchStore {
  pub fn new(paths: &[String]) -> Self {
    let mut groups = HashMap::new();

    // Because match groups can imports other match groups,
    // we have to load them recursively starting from the
    // top-level ones.
    load_match_groups_recursively(&mut groups, paths);

    Self { groups }
  }
}

impl MatchStore for DefaultMatchStore {
  fn query(&self, paths: &[String]) -> MatchSet {
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
          let imports = group.imports.clone();
          groups.insert(path.clone(), group);

          load_match_groups_recursively(groups, &imports);
        }
        Err(error) => {
          error!("unable to load match group: {:?}", error);
        }
      }
    }
  }
}

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
      visited_paths.insert(path.clone());

      if let Some(group) = groups.get(path) {
        query_matches_for_paths(
          groups,
          visited_paths,
          visited_matches,
          visited_global_vars,
          matches,
          global_vars,
          &group.imports,
        );

        for m in group.matches.iter() {
          if !visited_matches.contains(&m.id) {
            matches.push(m);
            visited_matches.insert(m.id);
          }
        }

        for var in group.global_vars.iter() {
          if !visited_global_vars.contains(&var.id) {
            global_vars.push(var);
            visited_global_vars.insert(var.id);
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    matches::{MatchCause, MatchEffect, TextEffect, TriggerCause},
    util::tests::use_test_directory,
  };
  use std::fs::create_dir_all;

  fn create_match(trigger: &str, replace: &str) -> Match {
    Match {
      cause: MatchCause::Trigger(TriggerCause {
        triggers: vec![trigger.to_string()],
        ..Default::default()
      }),
      effect: MatchEffect::Text(TextEffect {
        replace: replace.to_string(),
        ..Default::default()
      }),
      ..Default::default()
    }
  }

  fn create_matches(matches: &[(&str, &str)]) -> Vec<Match> {
    matches
      .iter()
      .map(|(trigger, replace)| create_match(trigger, replace))
      .collect()
  }

  fn create_test_var(name: &str) -> Variable {
    Variable {
      name: name.to_string(),
      var_type: "test".to_string(),
      ..Default::default()
    }
  }

  fn create_vars(vars: &[&str]) -> Vec<Variable> {
    vars.iter().map(|var| create_test_var(var)).collect()
  }

  #[test]
  fn match_store_loads_correctly() {
    use_test_directory(|_, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(
        &base_file,
        r#"
      imports:
        - "_another.yml"
      
      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      let another_file = match_dir.join("_another.yml");
      std::fs::write(
        &another_file,
        r#"
      imports:
        - "sub/sub.yml"
      
      matches:
        - trigger: "hello"
          replace: "world2" 
        - trigger: "foo"
          replace: "bar"
      "#,
      )
      .unwrap();

      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(
        &sub_file,
        r#"
      matches:
        - trigger: "hello"
          replace: "world3" 
      "#,
      )
      .unwrap();

      let match_store = DefaultMatchStore::new(&[base_file.to_string_lossy().to_string()]);

      assert_eq!(match_store.groups.len(), 3);

      let base_group = &match_store
        .groups
        .get(&base_file.to_string_lossy().to_string())
        .unwrap()
        .matches;
      let base_group: Vec<Match> = base_group.iter().map(|m| {
        let mut copy = m.clone();
        copy.id = 0;
        copy
      }).collect();

      assert_eq!(base_group, create_matches(&[("hello", "world")]));

      let another_group = &match_store
        .groups
        .get(&another_file.to_string_lossy().to_string())
        .unwrap()
        .matches;
      let another_group: Vec<Match> = another_group.iter().map(|m| {
        let mut copy = m.clone();
        copy.id = 0;
        copy
      }).collect();
      assert_eq!(
        another_group,
        create_matches(&[("hello", "world2"), ("foo", "bar")])
      );

      let sub_group = &match_store
        .groups
        .get(&sub_file.to_string_lossy().to_string())
        .unwrap()
        .matches;
      let sub_group: Vec<Match> = sub_group.iter().map(|m| {
        let mut copy = m.clone();
        copy.id = 0;
        copy
      }).collect();
      assert_eq!(sub_group, create_matches(&[("hello", "world3")]));
    });
  }

  #[test]
  fn match_store_handles_circular_dependency() {
    use_test_directory(|_, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(
        &base_file,
        r#"
      imports:
        - "_another.yml"
      
      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      let another_file = match_dir.join("_another.yml");
      std::fs::write(
        &another_file,
        r#"
      imports:
        - "sub/sub.yml"
      
      matches:
        - trigger: "hello"
          replace: "world2" 
        - trigger: "foo"
          replace: "bar"
      "#,
      )
      .unwrap();

      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(
        &sub_file,
        r#"
      imports:
        - "../_another.yml"

      matches:
        - trigger: "hello"
          replace: "world3" 
      "#,
      )
      .unwrap();

      let match_store = DefaultMatchStore::new(&[base_file.to_string_lossy().to_string()]);

      assert_eq!(match_store.groups.len(), 3);
    });
  }

  #[test]
  fn match_store_query_single_path_with_imports() {
    use_test_directory(|_, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(
        &base_file,
        r#"
      imports:
        - "_another.yml"
      
      global_vars:
        - name: var1
          type: test

      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      let another_file = match_dir.join("_another.yml");
      std::fs::write(
        &another_file,
        r#"
      imports:
        - "sub/sub.yml"
      
      matches:
        - trigger: "hello"
          replace: "world2" 
        - trigger: "foo"
          replace: "bar"
      "#,
      )
      .unwrap();

      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(
        &sub_file,
        r#"
      global_vars:
        - name: var2
          type: test

      matches:
        - trigger: "hello"
          replace: "world3" 
      "#,
      )
      .unwrap();

      let match_store = DefaultMatchStore::new(&[base_file.to_string_lossy().to_string()]);

      let match_set = match_store.query(&[base_file.to_string_lossy().to_string()]);

      assert_eq!(
        match_set
          .matches
          .into_iter()
          .cloned()
          .map(|mut m| { m.id = 0; m })
          .collect::<Vec<Match>>(),
        create_matches(&[
          ("hello", "world3"),
          ("hello", "world2"),
          ("foo", "bar"),
          ("hello", "world"),
        ])
      );

      assert_eq!(
        match_set
          .global_vars
          .into_iter()
          .cloned()
          .map(|mut v| { v.id = 0; v })
          .collect::<Vec<Variable>>(),
        create_vars(&["var2", "var1"])
      );
    });
  }

  #[test]
  fn match_store_query_handles_circular_depencencies() {
    use_test_directory(|_, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(
        &base_file,
        r#"
      imports:
        - "_another.yml"
      
      global_vars:
        - name: var1
          type: test

      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      let another_file = match_dir.join("_another.yml");
      std::fs::write(
        &another_file,
        r#"
      imports:
        - "sub/sub.yml"
      
      matches:
        - trigger: "hello"
          replace: "world2" 
        - trigger: "foo"
          replace: "bar"
      "#,
      )
      .unwrap();

      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(
        &sub_file,
        r#"
      imports:
        - "../_another.yml"  # Circular import

      global_vars:
        - name: var2
          type: test

      matches:
        - trigger: "hello"
          replace: "world3" 
      "#,
      )
      .unwrap();

      let match_store = DefaultMatchStore::new(&[base_file.to_string_lossy().to_string()]);

      let match_set = match_store.query(&[base_file.to_string_lossy().to_string()]);

      assert_eq!(
        match_set
          .matches
          .into_iter()
          .cloned()
          .map(|mut m| { m.id = 0; m })
          .collect::<Vec<Match>>(),
        create_matches(&[
          ("hello", "world3"),
          ("hello", "world2"),
          ("foo", "bar"),
          ("hello", "world"),
        ])
      );

      assert_eq!(
        match_set
          .global_vars
          .into_iter()
          .cloned()
          .map(|mut v| { v.id = 0; v})
          .collect::<Vec<Variable>>(),
        create_vars(&["var2", "var1"])
      );
    });
  }

  #[test]
  fn match_store_query_multiple_paths() {
    use_test_directory(|_, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(
        &base_file,
        r#"
      imports:
        - "_another.yml"
      
      global_vars:
        - name: var1
          type: test

      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      let another_file = match_dir.join("_another.yml");
      std::fs::write(
        &another_file,
        r#"
      matches:
        - trigger: "hello"
          replace: "world2" 
        - trigger: "foo"
          replace: "bar"
      "#,
      )
      .unwrap();

      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(
        &sub_file,
        r#"
      global_vars:
        - name: var2
          type: test

      matches:
        - trigger: "hello"
          replace: "world3" 
      "#,
      )
      .unwrap();

      let match_store = DefaultMatchStore::new(&[
        base_file.to_string_lossy().to_string(),
        sub_file.to_string_lossy().to_string(),
      ]);

      let match_set = match_store.query(&[
        base_file.to_string_lossy().to_string(),
        sub_file.to_string_lossy().to_string(),
      ]);

      assert_eq!(
        match_set
          .matches
          .into_iter()
          .cloned()
          .map(|mut m| { m.id = 0; m })
          .collect::<Vec<Match>>(),
        create_matches(&[
          ("hello", "world2"),
          ("foo", "bar"),
          ("hello", "world"),
          ("hello", "world3"),
        ])
      );

      assert_eq!(
        match_set
          .global_vars
          .into_iter()
          .cloned()
          .map(|mut v| { v.id = 0; v })
          .collect::<Vec<Variable>>(),
        create_vars(&["var1", "var2"])
      );
    });
  }

  #[test]
  fn match_store_query_handle_duplicates_when_imports_and_paths_overlap() {
    use_test_directory(|_, match_dir, _| {
      let sub_dir = match_dir.join("sub");
      create_dir_all(&sub_dir).unwrap();

      let base_file = match_dir.join("base.yml");
      std::fs::write(
        &base_file,
        r#"
      imports:
        - "_another.yml"
      
      global_vars:
        - name: var1
          type: test

      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      let another_file = match_dir.join("_another.yml");
      std::fs::write(
        &another_file,
        r#"
      imports:
        - "sub/sub.yml"
      
      matches:
        - trigger: "hello"
          replace: "world2" 
        - trigger: "foo"
          replace: "bar"
      "#,
      )
      .unwrap();

      let sub_file = sub_dir.join("sub.yml");
      std::fs::write(
        &sub_file,
        r#"
      global_vars:
        - name: var2
          type: test

      matches:
        - trigger: "hello"
          replace: "world3" 
      "#,
      )
      .unwrap();

      let match_store = DefaultMatchStore::new(&[base_file.to_string_lossy().to_string()]);

      let match_set = match_store.query(&[
        base_file.to_string_lossy().to_string(),
        sub_file.to_string_lossy().to_string(),
      ]);

      assert_eq!(
        match_set
          .matches
          .into_iter()
          .cloned()
          .map(|mut m| { m.id = 0; m })
          .collect::<Vec<Match>>(),
        create_matches(&[
          ("hello", "world3"), // This appears only once, though it appears 2 times
          ("hello", "world2"),
          ("foo", "bar"),
          ("hello", "world"),
        ])
      );

      assert_eq!(
        match_set
          .global_vars
          .into_iter()
          .cloned()
          .map(|mut v| { v.id = 0; v })
          .collect::<Vec<Variable>>(),
        create_vars(&["var2", "var1"])
      );
    });
  }
}

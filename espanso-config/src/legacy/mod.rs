/*
 * This file is part of espanso.
 *
 * C title: (), class: (), exec: ()opyright (C) 2019-2021 Federico Terzi
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

use anyhow::Result;
use log::warn;
use regex::Regex;
use std::{collections::HashMap, path::Path, sync::Arc};

use self::config::LegacyConfig;
use crate::matches::{
  group::loader::yaml::{
    parse::{YAMLMatch, YAMLVariable},
    try_convert_into_match, try_convert_into_variable,
  },
  MatchEffect,
};
use crate::{config::store::DefaultConfigStore, counter::StructId};
use crate::{
  config::Config,
  config::{AppProperties, ConfigStore},
  counter::next_id,
  matches::{
    store::{MatchSet, MatchStore},
    Match, Variable,
  },
};
use std::convert::TryInto;

mod config;
mod model;

pub fn load(
  base_dir: &Path,
  package_dir: &Path,
) -> Result<(Box<dyn ConfigStore>, Box<dyn MatchStore>)> {
  let config_set = config::LegacyConfigSet::load(base_dir, package_dir)?;

  let mut match_deduplicate_map = HashMap::new();
  let mut var_deduplicate_map = HashMap::new();

  let (default_config, mut default_match_group) = split_config(config_set.default);
  deduplicate_ids(
    &mut default_match_group,
    &mut match_deduplicate_map,
    &mut var_deduplicate_map,
  );

  let mut match_groups = HashMap::new();
  match_groups.insert("default".to_string(), default_match_group);

  let mut custom_configs: Vec<Arc<dyn Config>> = Vec::new();
  for custom in config_set.specific {
    let (custom_config, mut custom_match_group) = split_config(custom);
    deduplicate_ids(
      &mut custom_match_group,
      &mut match_deduplicate_map,
      &mut var_deduplicate_map,
    );

    match_groups.insert(custom_config.name.clone(), custom_match_group);
    custom_configs.push(Arc::new(custom_config));
  }

  let config_store = DefaultConfigStore::from_configs(Arc::new(default_config), custom_configs);
  let match_store = LegacyMatchStore::new(match_groups);

  Ok((Box::new(config_store), Box::new(match_store)))
}

fn split_config(config: LegacyConfig) -> (LegacyInteropConfig, LegacyMatchGroup) {
  let global_vars = config
    .global_vars
    .iter()
    .filter_map(|var| {
      let var: YAMLVariable = serde_yaml::from_value(var.clone()).ok()?;
      let (var, warnings) = try_convert_into_variable(var, true).ok()?;
      for warning in warnings {
        warn!("{}", warning);
      }
      Some(var)
    })
    .collect();

  let matches = config
    .matches
    .iter()
    .filter_map(|var| {
      let m: YAMLMatch = serde_yaml::from_value(var.clone()).ok()?;
      let (m, warnings) = try_convert_into_match(m, true).ok()?;
      for warning in warnings {
        warn!("{}", warning);
      }
      Some(m)
    })
    .collect();

  let match_group = LegacyMatchGroup {
    matches,
    global_vars,
  };

  let config: LegacyInteropConfig = config.into();
  (config, match_group)
}

/// Due to the way the legacy configs are loaded (matches are copied multiple times in the various configs)
/// we need to deduplicate the ids of those matches (and global vars).
fn deduplicate_ids(
  match_group: &mut LegacyMatchGroup,
  match_map: &mut HashMap<Match, StructId>,
  var_map: &mut HashMap<Variable, StructId>,
) {
  deduplicate_vars(&mut match_group.global_vars, var_map);
  deduplicate_matches(&mut match_group.matches, match_map, var_map);
}

fn deduplicate_matches(
  matches: &mut [Match],
  match_map: &mut HashMap<Match, StructId>,
  var_map: &mut HashMap<Variable, StructId>,
) {
  for m in matches.iter_mut() {
    // Deduplicate variables first
    if let MatchEffect::Text(effect) = &mut m.effect {
      deduplicate_vars(&mut effect.vars, var_map);
    }

    let mut m_without_id = m.clone();
    m_without_id.id = 0;
    if let Some(id) = match_map.get(&m_without_id) {
      m.id = *id;
    } else {
      match_map.insert(m_without_id, m.id);
    }
  }
}

// TODO: test case of matches with inner variables
fn deduplicate_vars(vars: &mut [Variable], var_map: &mut HashMap<Variable, StructId>) {
  for v in vars.iter_mut() {
    let mut v_without_id = v.clone();
    v_without_id.id = 0;
    if let Some(id) = var_map.get(&v_without_id) {
      v.id = *id;
    } else {
      var_map.insert(v_without_id, v.id);
    }
  }
}

struct LegacyInteropConfig {
  pub name: String,
  match_paths: Vec<String>,

  id: i32,

  config: LegacyConfig,

  filter_title: Option<Regex>,
  filter_class: Option<Regex>,
  filter_exec: Option<Regex>,
}

impl From<config::LegacyConfig> for LegacyInteropConfig {
  fn from(config: config::LegacyConfig) -> Self {
    Self {
      id: next_id(),
      config: config.clone(),
      name: config.name.clone(),
      match_paths: vec![config.name],
      filter_title: if config.filter_title.is_empty() {
        None
      } else {
        Regex::new(&config.filter_title).ok()
      },
      filter_class: if config.filter_class.is_empty() {
        None
      } else {
        Regex::new(&config.filter_class).ok()
      },
      filter_exec: if config.filter_exec.is_empty() {
        None
      } else {
        Regex::new(&config.filter_exec).ok()
      },
    }
  }
}

impl Config for LegacyInteropConfig {
  fn id(&self) -> i32 {
    self.id
  }

  fn label(&self) -> &str {
    &self.config.name
  }

  fn backend(&self) -> crate::config::Backend {
    match self.config.backend {
      config::BackendType::Inject => crate::config::Backend::Inject,
      config::BackendType::Clipboard => crate::config::Backend::Clipboard,
      config::BackendType::Auto => crate::config::Backend::Auto,
    }
  }

  fn auto_restart(&self) -> bool {
    self.config.auto_restart
  }

  fn match_paths(&self) -> &[String] {
    &self.match_paths
  }

  fn is_match(&self, app: &AppProperties) -> bool {
    if self.filter_title.is_none() && self.filter_exec.is_none() && self.filter_class.is_none() {
      return false;
    }

    let is_title_match = if let Some(title_regex) = self.filter_title.as_ref() {
      if let Some(title) = app.title {
        title_regex.is_match(title)
      } else {
        false
      }
    } else {
      true
    };

    let is_exec_match = if let Some(exec_regex) = self.filter_exec.as_ref() {
      if let Some(exec) = app.exec {
        exec_regex.is_match(exec)
      } else {
        false
      }
    } else {
      true
    };

    let is_class_match = if let Some(class_regex) = self.filter_class.as_ref() {
      if let Some(class) = app.class {
        class_regex.is_match(class)
      } else {
        false
      }
    } else {
      true
    };

    // All the filters that have been specified must be true to define a match
    is_exec_match && is_title_match && is_class_match
  }

  fn clipboard_threshold(&self) -> usize {
    crate::config::default::DEFAULT_CLIPBOARD_THRESHOLD
  }

  fn pre_paste_delay(&self) -> usize {
    crate::config::default::DEFAULT_PRE_PASTE_DELAY
  }

  fn toggle_key(&self) -> Option<crate::config::ToggleKey> {
    match self.config.toggle_key {
      model::KeyModifier::CTRL => Some(crate::config::ToggleKey::Ctrl),
      model::KeyModifier::SHIFT => Some(crate::config::ToggleKey::Shift),
      model::KeyModifier::ALT => Some(crate::config::ToggleKey::Alt),
      model::KeyModifier::META => Some(crate::config::ToggleKey::Meta),
      model::KeyModifier::BACKSPACE => None,
      model::KeyModifier::OFF => None,
      model::KeyModifier::LEFT_CTRL => Some(crate::config::ToggleKey::LeftCtrl),
      model::KeyModifier::RIGHT_CTRL => Some(crate::config::ToggleKey::RightCtrl),
      model::KeyModifier::LEFT_ALT => Some(crate::config::ToggleKey::LeftAlt),
      model::KeyModifier::RIGHT_ALT => Some(crate::config::ToggleKey::RightAlt),
      model::KeyModifier::LEFT_META => Some(crate::config::ToggleKey::LeftMeta),
      model::KeyModifier::RIGHT_META => Some(crate::config::ToggleKey::RightMeta),
      model::KeyModifier::LEFT_SHIFT => Some(crate::config::ToggleKey::LeftShift),
      model::KeyModifier::RIGHT_SHIFT => Some(crate::config::ToggleKey::RightShift),
      model::KeyModifier::CAPS_LOCK => None,
    }
  }

  fn preserve_clipboard(&self) -> bool {
    self.config.preserve_clipboard
  }

  fn restore_clipboard_delay(&self) -> usize {
    self.config.restore_clipboard_delay.try_into().unwrap()
  }

  fn paste_shortcut_event_delay(&self) -> usize {
    crate::config::default::DEFAULT_SHORTCUT_EVENT_DELAY
  }

  fn paste_shortcut(&self) -> Option<String> {
    match self.config.paste_shortcut {
      model::PasteShortcut::Default => None,
      model::PasteShortcut::CtrlV => Some("CTRL+V".to_string()),
      model::PasteShortcut::CtrlShiftV => Some("CTRL+SHIFT+V".to_string()),
      model::PasteShortcut::ShiftInsert => Some("SHIFT+INSERT".to_string()),
      model::PasteShortcut::CtrlAltV => Some("CTRL+ALT+V".to_string()),
      model::PasteShortcut::MetaV => Some("META+V".to_string()),
    }
  }

  fn disable_x11_fast_inject(&self) -> bool {
    !self.config.fast_inject
  }

  fn inject_delay(&self) -> Option<usize> {
    if self.config.inject_delay == 0 {
      None
    } else {
      Some(self.config.inject_delay.try_into().unwrap())
    }
  }

  fn key_delay(&self) -> Option<usize> {
    if self.config.backspace_delay == 0 {
      None
    } else {
      Some(self.config.backspace_delay.try_into().unwrap())
    }
  }

  fn word_separators(&self) -> Vec<String> {
    self
      .config
      .word_separators
      .iter()
      .map(|c| String::from(*c))
      .collect()
  }

  fn backspace_limit(&self) -> usize {
    self.config.backspace_limit.try_into().unwrap()
  }

  fn apply_patch(&self) -> bool {
    true
  }

  fn keyboard_layout(&self) -> Option<crate::config::RMLVOConfig> {
    None
  }

  fn search_trigger(&self) -> Option<String> {
    self.config.search_trigger.clone()
  }

  fn search_shortcut(&self) -> Option<String> {
    self.config.search_shortcut.clone()
  }

  fn undo_backspace(&self) -> bool {
    self.config.undo_backspace
  }

  fn show_icon(&self) -> bool {
    self.config.show_icon
  }

  fn show_notifications(&self) -> bool {
    self.config.show_notifications
  }

  fn secure_input_notification(&self) -> bool {
    self.config.secure_input_notification
  }

  fn enable(&self) -> bool {
    self.config.enable_active
  }

  fn post_form_delay(&self) -> usize {
    crate::config::default::DEFAULT_POST_FORM_DELAY
  }

  fn post_search_delay(&self) -> usize {
    crate::config::default::DEFAULT_POST_SEARCH_DELAY
  }

  fn emulate_alt_codes(&self) -> bool {
    false
  }

  fn win32_exclude_orphan_events(&self) -> bool {
    true
  }

  fn evdev_modifier_delay(&self) -> Option<usize> {
    Some(10)
  }

  fn win32_keyboard_layout_cache_interval(&self) -> i64 {
    2000
  }

  fn x11_use_xclip_backend(&self) -> bool {
    false
  }

  fn x11_use_xdotool_backend(&self) -> bool {
    false
  }
}

struct LegacyMatchGroup {
  matches: Vec<Match>,
  global_vars: Vec<Variable>,
}

struct LegacyMatchStore {
  groups: HashMap<String, LegacyMatchGroup>,
}

impl LegacyMatchStore {
  pub fn new(groups: HashMap<String, LegacyMatchGroup>) -> Self {
    Self { groups }
  }
}

impl MatchStore for LegacyMatchStore {
  fn query(&self, paths: &[String]) -> MatchSet {
    let group = if paths.is_empty() {
      None
    } else {
      self.groups.get(&paths[0])
    };

    if let Some(group) = group {
      MatchSet {
        matches: group.matches.iter().collect(),
        global_vars: group.global_vars.iter().collect(),
      }
    } else {
      MatchSet {
        matches: Vec::new(),
        global_vars: Vec::new(),
      }
    }
  }

  fn loaded_paths(&self) -> Vec<String> {
    self.groups.keys().cloned().collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{fs::create_dir_all, path::Path};
  use tempdir::TempDir;

  pub fn use_test_directory(callback: impl FnOnce(&Path, &Path, &Path)) {
    let dir = TempDir::new("tempconfig").unwrap();
    let user_dir = dir.path().join("user");
    create_dir_all(&user_dir).unwrap();

    let package_dir = TempDir::new("tempconfig").unwrap();

    callback(
      &dunce::canonicalize(dir.path()).unwrap(),
      &dunce::canonicalize(&user_dir).unwrap(),
      &dunce::canonicalize(package_dir.path()).unwrap(),
    );
  }

  #[test]
  fn load_legacy_works_correctly() {
    use_test_directory(|base, user, packages| {
      std::fs::write(
        base.join("default.yml"),
        r#"
      backend: Clipboard

      global_vars:
        - name: var1
          type: test

      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      std::fs::write(
        user.join("specific.yml"),
        r#"
      name: specific
      parent: default

      matches:
        - trigger: "foo"
          replace: "bar"
      "#,
      )
      .unwrap();

      std::fs::write(
        user.join("separate.yml"),
        r#"
      name: separate
      filter_title: "Google"

      matches:
        - trigger: "eren"
          replace: "mikasa"
      "#,
      )
      .unwrap();

      let (config_store, match_store) = load(base, packages).unwrap();

      let default_config = config_store.default();
      assert_eq!(default_config.match_paths().len(), 1);

      let active_config = config_store.active(&AppProperties {
        title: Some("Google"),
        class: None,
        exec: None,
      });
      assert_eq!(active_config.match_paths().len(), 1);

      let default_fallback = config_store.active(&AppProperties {
        title: Some("Yahoo"),
        class: None,
        exec: None,
      });
      assert_eq!(default_fallback.match_paths().len(), 1);

      assert_eq!(
        match_store
          .query(default_config.match_paths())
          .matches
          .len(),
        2
      );
      assert_eq!(
        match_store
          .query(default_config.match_paths())
          .global_vars
          .len(),
        1
      );

      assert_eq!(
        match_store.query(active_config.match_paths()).matches.len(),
        3
      );
      assert_eq!(
        match_store
          .query(active_config.match_paths())
          .global_vars
          .len(),
        1
      );

      assert_eq!(
        match_store
          .query(default_fallback.match_paths())
          .matches
          .len(),
        2
      );
      assert_eq!(
        match_store
          .query(default_fallback.match_paths())
          .global_vars
          .len(),
        1
      );
    });
  }

  #[test]
  fn load_legacy_deduplicates_ids_correctly() {
    use_test_directory(|base, user, packages| {
      std::fs::write(
        base.join("default.yml"),
        r#"
      backend: Clipboard

      global_vars:
        - name: var1
          type: test

      matches:
        - trigger: "hello"
          replace: "world"
        
        - trigger: "withvars"
          replace: "{{output}}"
          vars:
            - name: "output"
              type: "echo"
              params:
                echo: "test"
      "#,
      )
      .unwrap();

      std::fs::write(
        user.join("specific.yml"),
        r#"
      name: specific
      filter_title: "Google"
      "#,
      )
      .unwrap();

      let (config_store, match_store) = load(base, packages).unwrap();

      let default_config = config_store.default();
      let active_config = config_store.active(&AppProperties {
        title: Some("Google"),
        class: None,
        exec: None,
      });

      for (i, m) in match_store
        .query(default_config.match_paths())
        .matches
        .into_iter()
        .enumerate()
      {
        assert_eq!(
          m.id,
          match_store
            .query(active_config.match_paths())
            .matches
            .get(i)
            .unwrap()
            .id
        );
      }

      assert_eq!(
        match_store
          .query(default_config.match_paths())
          .global_vars
          .first()
          .unwrap()
          .id,
        match_store
          .query(active_config.match_paths())
          .global_vars
          .first()
          .unwrap()
          .id,
      );
    });
  }

  #[test]
  fn load_legacy_with_packages() {
    use_test_directory(|base, _, packages| {
      std::fs::write(
        base.join("default.yml"),
        r#"
      backend: Clipboard

      matches:
        - trigger: "hello"
          replace: "world"
      "#,
      )
      .unwrap();

      create_dir_all(packages.join("test-package")).unwrap();
      std::fs::write(
        packages.join("test-package").join("package.yml"),
        r#"
      name: test-package 
      parent: default

      matches:
        - trigger: "foo"
          replace: "bar"
      "#,
      )
      .unwrap();

      let (config_store, match_store) = load(base, packages).unwrap();

      let default_config = config_store.default();
      assert_eq!(default_config.match_paths().len(), 1);

      assert_eq!(
        match_store
          .query(default_config.match_paths())
          .matches
          .len(),
        2
      );
    });
  }
}

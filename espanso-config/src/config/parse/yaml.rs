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

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::Mapping;
use std::convert::TryFrom;

use crate::util::is_yaml_empty;

use super::ParsedConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct YAMLConfig {
  #[serde(default)]
  pub label: Option<String>,

  #[serde(default)]
  pub backend: Option<String>,

  #[serde(default)]
  pub enable: Option<bool>,

  #[serde(default)]
  pub clipboard_threshold: Option<usize>,

  #[serde(default)]
  pub pre_paste_delay: Option<usize>,

  #[serde(default)]
  pub toggle_key: Option<String>,

  #[serde(default)]
  pub auto_restart: Option<bool>,

  #[serde(default)]
  pub preserve_clipboard: Option<bool>,

  #[serde(default)]
  pub restore_clipboard_delay: Option<usize>,

  #[serde(default)]
  pub paste_shortcut_event_delay: Option<usize>,

  #[serde(default)]
  pub paste_shortcut: Option<String>,

  #[serde(default)]
  pub disable_x11_fast_inject: Option<bool>,

  #[serde(default)]
  pub inject_delay: Option<usize>,

  #[serde(default)]
  pub key_delay: Option<usize>,

  #[serde(default)]
  pub backspace_delay: Option<usize>,

  #[serde(default)]
  pub evdev_modifier_delay: Option<usize>,

  #[serde(default)]
  pub word_separators: Option<Vec<String>>,

  #[serde(default)]
  pub backspace_limit: Option<usize>,

  #[serde(default)]
  pub apply_patch: Option<bool>,

  #[serde(default)]
  pub keyboard_layout: Option<Mapping>,

  #[serde(default)]
  pub search_trigger: Option<String>,

  #[serde(default)]
  pub search_shortcut: Option<String>,

  #[serde(default)]
  pub undo_backspace: Option<bool>,

  #[serde(default)]
  pub show_notifications: Option<bool>,

  #[serde(default)]
  pub show_icon: Option<bool>,

  #[serde(default)]
  pub post_form_delay: Option<usize>,

  #[serde(default)]
  pub max_form_width: Option<usize>,

  #[serde(default)]
  pub max_form_height: Option<usize>,

  #[serde(default)]
  pub post_search_delay: Option<usize>,

  #[serde(default)]
  pub secure_input_notification: Option<bool>,

  #[serde(default)]
  pub emulate_alt_codes: Option<bool>,

  #[serde(default)]
  pub win32_exclude_orphan_events: Option<bool>,

  #[serde(default)]
  pub win32_keyboard_layout_cache_interval: Option<i64>,

  #[serde(default)]
  pub x11_use_xclip_backend: Option<bool>,

  #[serde(default)]
  pub x11_use_xdotool_backend: Option<bool>,

  // Include/Exclude
  #[serde(default)]
  pub includes: Option<Vec<String>>,

  #[serde(default)]
  pub excludes: Option<Vec<String>>,

  #[serde(default)]
  pub extra_includes: Option<Vec<String>>,

  #[serde(default)]
  pub extra_excludes: Option<Vec<String>>,

  #[serde(default)]
  pub use_standard_includes: Option<bool>,

  // Filters
  #[serde(default)]
  pub filter_title: Option<String>,

  #[serde(default)]
  pub filter_class: Option<String>,

  #[serde(default)]
  pub filter_exec: Option<String>,

  #[serde(default)]
  pub filter_os: Option<String>,
}

impl YAMLConfig {
  pub fn parse_from_str(yaml: &str) -> Result<Self> {
    // Because an empty string is not valid YAML but we want to support it anyway
    if is_yaml_empty(yaml) {
      return Ok(serde_yaml::from_str(
        "arbitrary_field_that_will_not_block_the_parser: true",
      )?);
    }

    Ok(serde_yaml::from_str(yaml)?)
  }
}

impl TryFrom<YAMLConfig> for ParsedConfig {
  type Error = anyhow::Error;

  fn try_from(yaml_config: YAMLConfig) -> Result<Self, Self::Error> {
    Ok(Self {
      label: yaml_config.label,
      backend: yaml_config.backend,
      enable: yaml_config.enable,
      clipboard_threshold: yaml_config.clipboard_threshold,
      auto_restart: yaml_config.auto_restart,
      toggle_key: yaml_config.toggle_key,
      preserve_clipboard: yaml_config.preserve_clipboard,
      paste_shortcut: yaml_config.paste_shortcut,
      disable_x11_fast_inject: yaml_config.disable_x11_fast_inject,
      inject_delay: yaml_config.inject_delay,
      key_delay: yaml_config.key_delay.or(yaml_config.backspace_delay),
      evdev_modifier_delay: yaml_config.evdev_modifier_delay,
      word_separators: yaml_config.word_separators,
      backspace_limit: yaml_config.backspace_limit,
      apply_patch: yaml_config.apply_patch,
      keyboard_layout: yaml_config.keyboard_layout.map(|mapping| {
        mapping
          .into_iter()
          .filter_map(|(key, value)| {
            if let (Some(key), Some(value)) = (key.as_str(), value.as_str()) {
              Some((key.to_string(), value.to_string()))
            } else {
              None
            }
          })
          .collect()
      }),
      search_trigger: yaml_config.search_trigger,
      search_shortcut: yaml_config.search_shortcut,
      undo_backspace: yaml_config.undo_backspace,

      show_icon: yaml_config.show_icon,
      show_notifications: yaml_config.show_notifications,
      secure_input_notification: yaml_config.secure_input_notification,

      pre_paste_delay: yaml_config.pre_paste_delay,
      restore_clipboard_delay: yaml_config.restore_clipboard_delay,
      paste_shortcut_event_delay: yaml_config.paste_shortcut_event_delay,
      post_form_delay: yaml_config.post_form_delay,
      max_form_width: yaml_config.max_form_width,
      max_form_height: yaml_config.max_form_height,
      post_search_delay: yaml_config.post_search_delay,

      emulate_alt_codes: yaml_config.emulate_alt_codes,

      win32_exclude_orphan_events: yaml_config.win32_exclude_orphan_events,
      win32_keyboard_layout_cache_interval: yaml_config.win32_keyboard_layout_cache_interval,
      x11_use_xclip_backend: yaml_config.x11_use_xclip_backend,
      x11_use_xdotool_backend: yaml_config.x11_use_xdotool_backend,

      use_standard_includes: yaml_config.use_standard_includes,
      includes: yaml_config.includes,
      extra_includes: yaml_config.extra_includes,
      excludes: yaml_config.excludes,
      extra_excludes: yaml_config.extra_excludes,

      filter_class: yaml_config.filter_class,
      filter_exec: yaml_config.filter_exec,
      filter_os: yaml_config.filter_os,
      filter_title: yaml_config.filter_title,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{collections::BTreeMap, convert::TryInto};

  #[test]
  fn conversion_to_parsed_config_works_correctly() {
    let config = YAMLConfig::parse_from_str(
      r#"
    label: "test"
    backend: clipboard
    enable: false
    clipboard_threshold: 200
    pre_paste_delay: 300
    toggle_key: CTRL
    auto_restart: false
    preserve_clipboard: false
    restore_clipboard_delay: 400
    paste_shortcut: CTRL+ALT+V
    paste_shortcut_event_delay: 10
    disable_x11_fast_inject: true
    inject_delay: 10
    key_delay: 20
    backspace_delay: 30
    evdev_modifier_delay: 40
    word_separators: ["'", "."]
    backspace_limit: 10
    apply_patch: false
    keyboard_layout:
      rules: test_rule
      model: test_model
      layout: test_layout
      variant: test_variant
      options: test_options
    search_trigger: "search"
    search_shortcut: "CTRL+SPACE"
    undo_backspace: false
    show_icon: false
    show_notifications: false
    secure_input_notification: false
    post_form_delay: 300
    max_form_width: 700
    max_form_height: 500
    post_search_delay: 400
    emulate_alt_codes: true
    win32_exclude_orphan_events: false
    win32_keyboard_layout_cache_interval: 300
    x11_use_xclip_backend: true
    x11_use_xdotool_backend: true

    use_standard_includes: true
    includes: ["test1"]
    extra_includes: ["test2"]
    excludes: ["test3"]
    extra_excludes: ["test4"]

    filter_class: "test5"
    filter_exec: "test6"
    filter_os: "test7"
    filter_title: "test8"
    "#,
    )
    .unwrap();
    let parsed_config: ParsedConfig = config.try_into().unwrap();

    let keyboard_layout: BTreeMap<String, String> = vec![
      ("rules".to_string(), "test_rule".to_string()),
      ("model".to_string(), "test_model".to_string()),
      ("layout".to_string(), "test_layout".to_string()),
      ("variant".to_string(), "test_variant".to_string()),
      ("options".to_string(), "test_options".to_string()),
    ]
    .into_iter()
    .collect();

    assert_eq!(
      parsed_config,
      ParsedConfig {
        label: Some("test".to_string()),

        backend: Some("clipboard".to_string()),
        enable: Some(false),
        clipboard_threshold: Some(200),
        auto_restart: Some(false),
        preserve_clipboard: Some(false),
        restore_clipboard_delay: Some(400),
        paste_shortcut: Some("CTRL+ALT+V".to_string()),
        paste_shortcut_event_delay: Some(10),
        disable_x11_fast_inject: Some(true),
        inject_delay: Some(10),
        key_delay: Some(20),
        backspace_limit: Some(10),
        apply_patch: Some(false),
        keyboard_layout: Some(keyboard_layout),
        search_trigger: Some("search".to_owned()),
        search_shortcut: Some("CTRL+SPACE".to_owned()),
        undo_backspace: Some(false),
        show_icon: Some(false),
        show_notifications: Some(false),
        secure_input_notification: Some(false),
        emulate_alt_codes: Some(true),
        post_form_delay: Some(300),
        max_form_width: Some(700),
        max_form_height: Some(500),
        post_search_delay: Some(400),
        win32_exclude_orphan_events: Some(false),
        win32_keyboard_layout_cache_interval: Some(300),
        x11_use_xclip_backend: Some(true),
        x11_use_xdotool_backend: Some(true),

        pre_paste_delay: Some(300),
        evdev_modifier_delay: Some(40),

        toggle_key: Some("CTRL".to_string()),
        word_separators: Some(vec!["'".to_owned(), ".".to_owned()]),

        use_standard_includes: Some(true),
        includes: Some(vec!["test1".to_string()]),
        extra_includes: Some(vec!["test2".to_string()]),
        excludes: Some(vec!["test3".to_string()]),
        extra_excludes: Some(vec!["test4".to_string()]),

        filter_class: Some("test5".to_string()),
        filter_exec: Some("test6".to_string()),
        filter_os: Some("test7".to_string()),
        filter_title: Some("test8".to_string()),
      }
    );
  }
}

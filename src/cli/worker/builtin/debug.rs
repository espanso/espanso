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

use espanso_engine::event::{effect::TextInjectRequest, ui::ShowTextEvent, EventType};

use crate::cli::worker::builtin::generate_next_builtin_id;

use super::BuiltInMatch;

// TODO: create task that opens up a GUI with this content

pub fn create_match_paste_active_config_info() -> BuiltInMatch {
  BuiltInMatch {
    id: generate_next_builtin_id(),
    label: "Paste active config information",
    triggers: vec!["#pacfg#".to_string()],
    action: |context| {
      let dump = context.get_active_config().pretty_dump();

      EventType::TextInject(TextInjectRequest {
        text: dump,
        force_mode: None,
      })
    },
    ..Default::default()
  }
}

pub fn create_match_paste_active_app_info() -> BuiltInMatch {
  BuiltInMatch {
    id: generate_next_builtin_id(),
    label: "Paste active application information (detect)",
    triggers: vec!["#pdetect#".to_string()],
    action: |context| {
      let info = context.get_active_app_info();

      let dump = format!(
        "title: '{}'\nexec: '{}'\nclass: '{}'",
        info.title.unwrap_or_default(),
        info.exec.unwrap_or_default(),
        info.class.unwrap_or_default()
      );

      EventType::TextInject(TextInjectRequest {
        text: dump,
        force_mode: None,
      })
    },
    ..Default::default()
  }
}

pub fn create_match_show_active_config_info() -> BuiltInMatch {
  BuiltInMatch {
    id: generate_next_builtin_id(),
    label: "Show active config information",
    triggers: vec!["#acfg#".to_string()],
    action: |context| {
      let dump = context.get_active_config().pretty_dump();

      EventType::ShowText(ShowTextEvent {
        text: dump,
        title: "Active configuration".to_string(),
      })
    },
    ..Default::default()
  }
}

pub fn create_match_show_active_app_info() -> BuiltInMatch {
  BuiltInMatch {
    id: generate_next_builtin_id(),
    label: "Show active application information (detect)",
    triggers: vec!["#detect#".to_string()],
    action: |context| {
      let info = context.get_active_app_info();

      let dump = format!(
        "title: '{}'\nexec: '{}'\nclass: '{}'",
        info.title.unwrap_or_default(),
        info.exec.unwrap_or_default(),
        info.class.unwrap_or_default()
      );

      EventType::ShowText(ShowTextEvent {
        text: dump,
        title: "Active application information (detect)".to_string(),
      })
    },
    ..Default::default()
  }
}

pub fn create_match_show_logs() -> BuiltInMatch {
  BuiltInMatch {
    id: generate_next_builtin_id(),
    label: "Show Espanso's logs",
    triggers: vec!["#log#".to_string()],
    action: |_| EventType::ShowLogs,
    ..Default::default()
  }
}

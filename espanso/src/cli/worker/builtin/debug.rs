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

use crate::{cli::worker::builtin::generate_next_builtin_id, engine::event::{EventType, effect::{TextInjectRequest}}};

use super::BuiltInMatch;

// TODO: create task that opens up a GUI with this content

pub fn create_match_paste_active_config_info() -> BuiltInMatch {
  BuiltInMatch {
    id: generate_next_builtin_id(),
    label: "Paste active config information",
    triggers: vec!["#acfg#".to_string()],
    action: |context| {
      let dump = context.get_active_config().pretty_dump();

      EventType::TextInject(TextInjectRequest {
        text: dump,
        force_mode: None,
      })
    },
  }
}
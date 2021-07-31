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

use crate::{cli::worker::builtin::generate_next_builtin_id, engine::event::{EventType, effect::TextInjectRequest}};

use super::BuiltInMatch;

pub fn create_match_show_active_config_info() -> BuiltInMatch {
  BuiltInMatch {
    id: generate_next_builtin_id(),
    label: "Display active config information",
    triggers: vec!["#acfg#".to_string()],
    action: |context| {
      println!("active config: {:?}", context.get_active_config().label());

      EventType::TextInject(TextInjectRequest {
        text: "test".to_string(),
        force_mode: None,
      })
    },
  }
}
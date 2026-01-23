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

use espanso_engine::event::EventType;

use crate::cli::worker::builtin::generate_next_builtin_id;

use super::BuiltInMatch;

pub fn create_match_export_config() -> BuiltInMatch {
    BuiltInMatch {
        id: generate_next_builtin_id(),
        label: "Export config",
        triggers: Vec::new(),
        action: |_context| {
            // TODO: Launch export dialog via modulo
            EventType::NOOP
        },
        ..Default::default()
    }
}

pub fn create_match_import_config() -> BuiltInMatch {
    BuiltInMatch {
        id: generate_next_builtin_id(),
        label: "Import config",
        triggers: Vec::new(),
        action: |_context| {
            // TODO: Launch import dialog via modulo
            EventType::NOOP
        },
        ..Default::default()
    }
}

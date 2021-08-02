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

use std::sync::Arc;

use crate::patch::patches::{PatchedConfig, Patches};
use crate::patch::PatchDefinition;

pub fn patch() -> PatchDefinition {
  PatchDefinition {
    name: module_path!().split(':').last().unwrap_or("unknown"),
    is_enabled: || cfg!(target_os = "linux") && !super::util::is_wayland(),
    should_patch: |app| app.class.unwrap_or_default().contains("terminal"),
    apply: |base, name| {
      Arc::new(PatchedConfig::patch(
        base,
        name,
        Patches {
          paste_shortcut: Some(Some("CTRL+SHIFT+V".to_string())),
          ..Default::default()
        },
      ))
    },
  }
}
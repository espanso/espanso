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

use espanso_config::config::{Backend, RMLVOConfig, ToggleKey};

#[cfg(target_os = "windows")]
pub mod win;

#[cfg(target_os = "linux")]
pub mod linux;

#[macro_use]
mod macros;

generate_patchable_config!(
  PatchedConfig,
  backend -> Backend,
  enable -> bool,
  clipboard_threshold -> usize,
  pre_paste_delay -> usize,
  paste_shortcut_event_delay -> usize,
  paste_shortcut -> Option<String>,
  disable_x11_fast_inject -> bool,
  toggle_key -> Option<ToggleKey>,
  auto_restart -> bool,
  preserve_clipboard -> bool,
  restore_clipboard_delay -> usize,
  inject_delay -> Option<usize>,
  key_delay -> Option<usize>,
  evdev_modifier_delay -> Option<usize>,
  word_separators -> Vec<String>,
  backspace_limit -> usize,
  apply_patch -> bool,
  undo_backspace -> bool,
  post_form_delay -> usize,
  post_search_delay -> usize,
  win32_exclude_orphan_events -> bool,
  win32_keyboard_layout_cache_interval -> i64,
  x11_use_xclip_backend -> bool,
  x11_use_xdotool_backend -> bool,
  keyboard_layout -> Option<RMLVOConfig>
);

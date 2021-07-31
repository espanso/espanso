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

use espanso_config::config::{AppProperties, Backend, Config, ToggleKey};

mod config_store;
mod patches;

pub fn get_builtin_patches() -> Vec<PatchDefinition> {
  // TODO
  vec![
    patches::win::onenote_for_windows_10::patch(),
  ]
}

// TODO: fix visibility levels (pub/pub(crate))

// TODO: patches should be "merged" at runtime with the active config, unless a config option (like "apply_patch")
// is set. This is needed because we still want to allow the user to define user-specific configs without
// losing the patch-specific changes

pub struct PatchDefinition {
  pub name: &'static str,
  pub should_be_activated: fn() -> bool,
  pub patch_config: fn(config: Arc<dyn Config>) -> Arc<dyn PatchedConfig>,
}

pub struct DefaultPatchedConfig {
  base: Arc<dyn Config>,
}

pub trait PatchedConfig: Config {
  // TODO: can we pass a simple reference here?
  fn get_base(&self) -> Arc<dyn Config>;

  fn id(&self) -> i32 {
    self.get_base().id()
  }

  fn label(&self) -> &str {
    self.get_base().label()
  }

  fn match_paths(&self) -> &[String] {
    self.get_base().match_paths()
  }

  fn backend(&self) -> Backend {
    self.get_base().backend()
  }

  fn clipboard_threshold(&self) -> usize {
    self.get_base().clipboard_threshold()
  }

  fn pre_paste_delay(&self) -> usize {
    self.get_base().pre_paste_delay()
  }

  fn paste_shortcut_event_delay(&self) -> usize {
    self.get_base().paste_shortcut_event_delay()
  }

  fn paste_shortcut(&self) -> Option<String> {
    self.get_base().paste_shortcut()
  }

  fn disable_x11_fast_inject(&self) -> bool {
    self.get_base().disable_x11_fast_inject()
  }

  fn toggle_key(&self) -> Option<ToggleKey> {
    self.get_base().toggle_key()
  }

  fn auto_restart(&self) -> bool {
    self.get_base().auto_restart()
  }

  fn preserve_clipboard(&self) -> bool {
    self.get_base().preserve_clipboard()
  }

  fn restore_clipboard_delay(&self) -> usize {
    self.get_base().restore_clipboard_delay()
  }

  fn inject_delay(&self) -> Option<usize> {
    self.get_base().inject_delay()
  }

  fn key_delay(&self) -> Option<usize> {
    self.get_base().key_delay()
  }

  fn word_separators(&self) -> Vec<String> {
    self.get_base().word_separators()
  }

  fn backspace_limit(&self) -> usize {
    self.get_base().backspace_limit()
  }

  fn is_match<'a>(&self, app: &AppProperties<'a>) -> bool {
    self.get_base().is_match(app)
  }
}

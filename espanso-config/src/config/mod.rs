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
use indoc::formatdoc;
use std::sync::Arc;
use std::{collections::HashSet, path::Path};
use thiserror::Error;

pub(crate) mod default;
mod parse;
mod path;
mod resolve;
pub(crate) mod store;
mod util;

#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::error::NonFatalErrorSet;
#[cfg_attr(test, automock)]
pub trait Config: Send + Sync {
  fn id(&self) -> i32;
  fn label(&self) -> &str;
  fn match_paths(&self) -> &[String];
  fn backend(&self) -> Backend;

  // Number of chars after which a match is injected with the clipboard
  // backend instead of the default one. This is done for efficiency
  // reasons, as injecting a long match through separate events becomes
  // slow for long strings.
  fn clipboard_threshold(&self) -> usize;

  // Delay (in ms) that espanso should wait to trigger the paste shortcut
  // after copying the content in the clipboard. This is needed because
  // if we trigger a "paste" shortcut before the content is actually
  // copied in the clipboard, the operation will fail.
  fn pre_paste_delay(&self) -> usize;

  // Number of milliseconds between keystrokes when simulating the Paste shortcut
  // For example: CTRL + (wait 5ms) + V + (wait 5ms) + release V + (wait 5ms) + release CTRL
  // This is needed as sometimes (for example on macOS), without a delay some keystrokes
  // were not registered correctly
  fn paste_shortcut_event_delay(&self) -> usize;

  // Customize the keyboard shortcut used to paste an expansion.
  // This should follow this format: CTRL+SHIFT+V
  fn paste_shortcut(&self) -> Option<String>;

  // NOTE: This is only relevant on Linux under X11 environments
  // Switch to a slower (but sometimes more supported) way of injecting
  // key events based on XTestFakeKeyEvent instead of XSendEvent.
  // From my experiements, disabling fast inject becomes particularly slow when
  // using the Gnome desktop environment.
  fn disable_x11_fast_inject(&self) -> bool;

  // Defines the key that disables/enables espanso when double pressed
  fn toggle_key(&self) -> Option<ToggleKey>;

  // If true, instructs the daemon process to restart the worker (and refresh
  // the configuration) after a configuration file change is detected on disk.
  fn auto_restart(&self) -> bool;

  // If true, espanso will attempt to preserve the previous clipboard content
  // after an expansion has taken place (when using the Clipboard backend).
  fn preserve_clipboard(&self) -> bool;

  // The number of milliseconds to wait before restoring the previous clipboard
  // content after an expansion. This is needed as without this delay, sometimes
  // the target application detects the previous clipboard content instead of
  // the expansion content.
  fn restore_clipboard_delay(&self) -> usize;

  // Number of milliseconds between text injection events. Increase if the target
  // application is missing some characters.
  fn inject_delay(&self) -> Option<usize>;

  // Number of milliseconds between key injection events. Increase if the target
  // application is missing some key events.
  fn key_delay(&self) -> Option<usize>;

  // Chars that when pressed mark the start and end of a word.
  // Examples of this are . or ,
  fn word_separators(&self) -> Vec<String>;

  // Maximum number of backspace presses espanso keeps track of.
  // For example, this is needed to correctly expand even if typos
  // are typed.
  fn backspace_limit(&self) -> usize;

  // If false, avoid applying the built-in patches to the current config.
  fn apply_patch(&self) -> bool;

  // On Wayland, overrides the auto-detected keyboard configuration (RMLVO)
  // which is used both for the detection and injection process.
  fn keyboard_layout(&self) -> Option<RMLVOConfig>;

  // Trigger used to show the Search UI
  fn search_trigger(&self) -> Option<String>;

  // Hotkey used to trigger the Search UI
  fn search_shortcut(&self) -> Option<String>;

  fn is_match<'a>(&self, app: &AppProperties<'a>) -> bool;

  fn pretty_dump(&self) -> String {
    formatdoc! {"
        [espanso config: {:?}]

        backend: {:?}
        paste_shortcut: {:?}
        inject_delay: {:?}
        key_delay: {:?}
        apply_patch: {:?}
        word_separators: {:?}
        
        preserve_clipboard: {:?}
        clipboard_threshold: {:?}
        disable_x11_fast_inject: {}
        pre_paste_delay: {}
        paste_shortcut_event_delay: {}
        toggle_key: {:?}
        auto_restart: {:?}
        restore_clipboard_delay: {:?} 
        backspace_limit: {}

        match_paths: {:#?}
      ", 
      self.label(),
      self.backend(),
      self.paste_shortcut(),
      self.inject_delay(),
      self.key_delay(),
      self.apply_patch(),
      self.word_separators(),

      self.preserve_clipboard(),
      self.clipboard_threshold(),
      self.disable_x11_fast_inject(),
      self.pre_paste_delay(),
      self.paste_shortcut_event_delay(),
      self.toggle_key(),
      self.auto_restart(),
      self.restore_clipboard_delay(),
      self.backspace_limit(),

      self.match_paths(),
    }
  }
}

pub trait ConfigStore: Send {
  fn default(&self) -> Arc<dyn Config>;
  fn active<'a>(&'a self, app: &AppProperties) -> Arc<dyn Config>;
  fn configs(&self) -> Vec<Arc<dyn Config>>;

  fn get_all_match_paths(&self) -> HashSet<String>;
}

pub struct AppProperties<'a> {
  pub title: Option<&'a str>,
  pub class: Option<&'a str>,
  pub exec: Option<&'a str>,
}

#[derive(Debug, Copy, Clone)]
pub enum Backend {
  Inject,
  Clipboard,
  Auto,
}

#[derive(Debug, Copy, Clone)]
pub enum ToggleKey {
  Ctrl,
  Meta,
  Alt,
  Shift,
  RightCtrl,
  RightAlt,
  RightShift,
  RightMeta,
  LeftCtrl,
  LeftAlt,
  LeftShift,
  LeftMeta,
}

#[derive(Debug, Clone, Default)]
pub struct RMLVOConfig {
  pub rules: Option<String>,
  pub model: Option<String>,
  pub layout: Option<String>,
  pub variant: Option<String>,
  pub options: Option<String>,
}

impl std::fmt::Display for RMLVOConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "[R={}, M={}, L={}, V={}, O={}]",
      self.rules.as_deref().unwrap_or_default(),
      self.model.as_deref().unwrap_or_default(),
      self.layout.as_deref().unwrap_or_default(),
      self.variant.as_deref().unwrap_or_default(),
      self.options.as_deref().unwrap_or_default(),
    )
  }
}

pub fn load_store(config_dir: &Path) -> Result<(impl ConfigStore, Vec<NonFatalErrorSet>)> {
  store::DefaultConfigStore::load(config_dir)
}

#[derive(Error, Debug)]
pub enum ConfigStoreError {
  #[error("invalid config directory")]
  InvalidConfigDir(),

  #[error("missing default.yml config")]
  MissingDefault(),

  #[error("io error")]
  IOError(#[from] std::io::Error),
}

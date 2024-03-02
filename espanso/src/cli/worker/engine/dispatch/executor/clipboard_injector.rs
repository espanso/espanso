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

use std::{convert::TryInto, path::PathBuf};

use espanso_clipboard::{Clipboard, ClipboardOperationOptions};
use espanso_inject::{keys::Key, InjectionOptions, Injector};
use log::error;

use espanso_engine::{
  dispatch::HtmlInjector,
  dispatch::{ImageInjector, TextInjector},
};

pub trait ClipboardParamsProvider {
  fn get(&self) -> ClipboardParams;
}

pub struct ClipboardParams {
  pub pre_paste_delay: usize,
  pub paste_shortcut_event_delay: usize,
  pub paste_shortcut: Option<String>,
  pub disable_x11_fast_inject: bool,
  pub restore_clipboard: bool,
  pub restore_clipboard_delay: usize,
  pub x11_use_xclip_backend: bool,
  pub x11_use_xdotool_backend: bool,
}

pub struct ClipboardInjectorAdapter<'a> {
  injector: &'a dyn Injector,
  clipboard: &'a dyn Clipboard,
  params_provider: &'a dyn ClipboardParamsProvider,
}

impl<'a> ClipboardInjectorAdapter<'a> {
  pub fn new(
    injector: &'a dyn Injector,
    clipboard: &'a dyn Clipboard,
    params_provider: &'a dyn ClipboardParamsProvider,
  ) -> Self {
    Self {
      injector,
      clipboard,
      params_provider,
    }
  }

  fn send_paste_combination(&self) -> anyhow::Result<()> {
    let params = self.params_provider.get();

    std::thread::sleep(std::time::Duration::from_millis(
      params.pre_paste_delay.try_into().unwrap(),
    ));

    let mut custom_combination = None;
    if let Some(custom_shortcut) = params.paste_shortcut {
      if let Some(combination) = parse_combination(&custom_shortcut) {
        custom_combination = Some(combination);
      } else {
        error!("'{}' is not a valid paste shortcut", custom_shortcut);
      }
    }

    let combination = if let Some(custom_combination) = custom_combination {
      custom_combination
    } else if cfg!(target_os = "macos") {
      vec![Key::Meta, Key::V]
    } else if cfg!(target_os = "linux") && cfg!(feature = "wayland") {
      // Because on Wayland we currently don't have app-specific configs (and therefore no patches)
      // we switch to the more supported SHIFT+INSERT combination
      // See: https://github.com/espanso/espanso/issues/899
      vec![Key::Shift, Key::Insert]
    } else {
      vec![Key::Control, Key::V]
    };

    self.injector.send_key_combination(
      &combination,
      InjectionOptions {
        delay: params.paste_shortcut_event_delay as i32,
        disable_fast_inject: params.disable_x11_fast_inject,
        x11_use_xdotool_fallback: params.x11_use_xdotool_backend,
        ..Default::default()
      },
    )?;

    Ok(())
  }

  fn restore_clipboard_guard(&self) -> Option<ClipboardRestoreGuard<'a>> {
    let params = self.params_provider.get();

    if params.restore_clipboard {
      Some(ClipboardRestoreGuard::lock(
        self.clipboard,
        params.restore_clipboard_delay.try_into().unwrap(),
        self.get_operation_options(),
      ))
    } else {
      None
    }
  }

  fn get_operation_options(&self) -> ClipboardOperationOptions {
    let params = self.params_provider.get();
    ClipboardOperationOptions {
      use_xclip_backend: params.x11_use_xclip_backend,
    }
  }
}

impl<'a> TextInjector for ClipboardInjectorAdapter<'a> {
  fn name(&self) -> &'static str {
    "clipboard"
  }

  fn inject_text(&self, text: &str) -> anyhow::Result<()> {
    let _guard = self.restore_clipboard_guard();

    self
      .clipboard
      .set_text(text, &self.get_operation_options())?;

    self.send_paste_combination()?;

    Ok(())
  }
}

impl<'a> HtmlInjector for ClipboardInjectorAdapter<'a> {
  fn inject_html(&self, html: &str, fallback_text: &str) -> anyhow::Result<()> {
    let _guard = self.restore_clipboard_guard();

    self
      .clipboard
      .set_html(html, Some(fallback_text), &self.get_operation_options())?;

    self.send_paste_combination()?;

    Ok(())
  }
}

impl<'a> ImageInjector for ClipboardInjectorAdapter<'a> {
  fn inject_image(&self, image_path: &str) -> anyhow::Result<()> {
    let path = PathBuf::from(image_path);
    if !path.is_file() {
      return Err(
        std::io::Error::new(
          std::io::ErrorKind::NotFound,
          "image can't be found in the given path",
        )
        .into(),
      );
    }

    let _guard = self.restore_clipboard_guard();

    self
      .clipboard
      .set_image(&path, &self.get_operation_options())?;

    self.send_paste_combination()?;

    Ok(())
  }
}

struct ClipboardRestoreGuard<'a> {
  clipboard: &'a dyn Clipboard,
  content: Option<String>,
  restore_delay: u64,
  clipboard_operation_options: ClipboardOperationOptions,
}

impl<'a> ClipboardRestoreGuard<'a> {
  pub fn lock(
    clipboard: &'a dyn Clipboard,
    restore_delay: u64,
    clipboard_operation_options: ClipboardOperationOptions,
  ) -> Self {
    let clipboard_content = clipboard.get_text(&clipboard_operation_options);

    Self {
      clipboard,
      content: clipboard_content,
      restore_delay,
      clipboard_operation_options,
    }
  }
}

impl<'a> Drop for ClipboardRestoreGuard<'a> {
  fn drop(&mut self) {
    if let Some(content) = self.content.take() {
      // Sometimes an expansion gets overwritten before pasting by the previous content
      // A delay is needed to mitigate the problem
      std::thread::sleep(std::time::Duration::from_millis(self.restore_delay));

      if let Err(error) = self
        .clipboard
        .set_text(&content, &self.clipboard_operation_options)
      {
        error!(
          "unable to restore clipboard content after expansion: {}",
          error
        );
      }
    }
  }
}

fn parse_combination(combination: &str) -> Option<Vec<Key>> {
  let tokens = combination.split('+');
  let mut keys: Vec<Key> = Vec::new();
  for token in tokens {
    keys.push(Key::parse(token)?);
  }

  Some(keys)
}

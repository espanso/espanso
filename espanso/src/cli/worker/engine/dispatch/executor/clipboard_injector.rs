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

use espanso_clipboard::Clipboard;
use espanso_inject::{keys::Key, InjectionOptions, Injector};

use crate::engine::{
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

    // TODO: handle case of custom combination
    let combination = if cfg!(target_os = "macos") {
      &[Key::Meta, Key::V]
    } else {
      &[Key::Control, Key::V]
    }; 

    // TODO: handle user-specified delays
    // let paste_combination_delay = if cfg!(target_os = "macos") {
    //   5
    // } else {
    //   InjectionOptions::default().delay
    // };

    // TODO: handle options
    self.injector.send_key_combination(
      combination,
      InjectionOptions {
        delay: params.paste_shortcut_event_delay as i32,
        ..Default::default()
      },
    )?;

    Ok(())
  }
}

impl<'a> TextInjector for ClipboardInjectorAdapter<'a> {
  fn name(&self) -> &'static str {
    "clipboard"
  }

  fn inject_text(&self, text: &str) -> anyhow::Result<()> {
    // TODO: handle clipboard restoration
    self.clipboard.set_text(text)?;

    self.send_paste_combination()?;

    Ok(())
  }
}

impl<'a> HtmlInjector for ClipboardInjectorAdapter<'a> {
  fn inject_html(&self, html: &str, fallback_text: &str) -> anyhow::Result<()> {
    // TODO: handle clipboard restoration
    self.clipboard.set_html(html, Some(fallback_text))?;

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

    // TODO: handle clipboard restoration
    self.clipboard.set_image(&path)?;

    self.send_paste_combination()?;

    Ok(())
  }
}

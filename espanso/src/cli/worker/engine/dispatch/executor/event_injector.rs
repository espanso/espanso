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

use std::convert::TryInto;

use espanso_inject::{InjectionOptions, Injector};

use espanso_engine::dispatch::TextInjector;

use super::InjectParamsProvider;

pub struct EventInjectorAdapter<'a> {
  injector: &'a dyn Injector,
  params_provider: &'a dyn InjectParamsProvider,
}

impl<'a> EventInjectorAdapter<'a> {
  pub fn new(injector: &'a dyn Injector, params_provider: &'a dyn InjectParamsProvider) -> Self {
    Self {
      injector,
      params_provider,
    }
  }
}

impl<'a> TextInjector for EventInjectorAdapter<'a> {
  fn name(&self) -> &'static str {
    "event"
  }

  fn inject_text(&self, text: &str) -> anyhow::Result<()> {
    let params = self.params_provider.get();

    // Handle CRLF or LF line endings correctly
    let split_sequence = if text.contains("\r\n") { "\r\n" } else { "\n" };

    let injection_options = InjectionOptions {
      delay: params
        .inject_delay
        .unwrap_or_else(|| InjectionOptions::default().delay.try_into().unwrap())
        .try_into()
        .unwrap(),
      disable_fast_inject: params.disable_x11_fast_inject,
      evdev_modifier_delay: params
        .evdev_modifier_delay
        .unwrap_or_else(|| {
          InjectionOptions::default()
            .evdev_modifier_delay
            .try_into()
            .unwrap()
        })
        .try_into()
        .unwrap(),
    };

    // We don't use the lines() method because it skips emtpy lines, which is not what we want.
    for (i, line) in text.split(split_sequence).enumerate() {
      // We simulate an Return press between lines
      if i > 0 {
        self
          .injector
          .send_keys(&[espanso_inject::keys::Key::Enter], injection_options)?
      }

      self.injector.send_string(line, injection_options)?;
    }

    Ok(())
  }
}

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

use espanso_inject::{InjectionOptions, Injector};
use std::convert::TryInto;

use espanso_engine::dispatch::KeyInjector;

use super::InjectParamsProvider;

pub struct KeyInjectorAdapter<'a> {
  injector: &'a dyn Injector,
  params_provider: &'a dyn InjectParamsProvider,
}

impl<'a> KeyInjectorAdapter<'a> {
  pub fn new(injector: &'a dyn Injector, params_provider: &'a dyn InjectParamsProvider) -> Self {
    Self {
      injector,
      params_provider,
    }
  }
}

impl<'a> KeyInjector for KeyInjectorAdapter<'a> {
  fn inject_sequence(&self, keys: &[espanso_engine::event::input::Key]) -> anyhow::Result<()> {
    let params = self.params_provider.get();

    let injection_options = InjectionOptions {
      delay: params
        .key_delay
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

    let converted_keys: Vec<_> = keys.iter().map(convert_to_inject_key).collect();
    self.injector.send_keys(&converted_keys, injection_options)
  }
}

fn convert_to_inject_key(key: &espanso_engine::event::input::Key) -> espanso_inject::keys::Key {
  match key {
    espanso_engine::event::input::Key::Alt => espanso_inject::keys::Key::Alt,
    espanso_engine::event::input::Key::CapsLock => espanso_inject::keys::Key::CapsLock,
    espanso_engine::event::input::Key::Control => espanso_inject::keys::Key::Control,
    espanso_engine::event::input::Key::Meta => espanso_inject::keys::Key::Meta,
    espanso_engine::event::input::Key::NumLock => espanso_inject::keys::Key::NumLock,
    espanso_engine::event::input::Key::Shift => espanso_inject::keys::Key::Shift,
    espanso_engine::event::input::Key::Enter => espanso_inject::keys::Key::Enter,
    espanso_engine::event::input::Key::Tab => espanso_inject::keys::Key::Tab,
    espanso_engine::event::input::Key::Space => espanso_inject::keys::Key::Space,
    espanso_engine::event::input::Key::ArrowDown => espanso_inject::keys::Key::ArrowDown,
    espanso_engine::event::input::Key::ArrowLeft => espanso_inject::keys::Key::ArrowLeft,
    espanso_engine::event::input::Key::ArrowRight => espanso_inject::keys::Key::ArrowRight,
    espanso_engine::event::input::Key::ArrowUp => espanso_inject::keys::Key::ArrowUp,
    espanso_engine::event::input::Key::End => espanso_inject::keys::Key::End,
    espanso_engine::event::input::Key::Home => espanso_inject::keys::Key::Home,
    espanso_engine::event::input::Key::PageDown => espanso_inject::keys::Key::PageDown,
    espanso_engine::event::input::Key::PageUp => espanso_inject::keys::Key::PageUp,
    espanso_engine::event::input::Key::Escape => espanso_inject::keys::Key::Escape,
    espanso_engine::event::input::Key::Backspace => espanso_inject::keys::Key::Backspace,
    espanso_engine::event::input::Key::F1 => espanso_inject::keys::Key::F1,
    espanso_engine::event::input::Key::F2 => espanso_inject::keys::Key::F2,
    espanso_engine::event::input::Key::F3 => espanso_inject::keys::Key::F3,
    espanso_engine::event::input::Key::F4 => espanso_inject::keys::Key::F4,
    espanso_engine::event::input::Key::F5 => espanso_inject::keys::Key::F5,
    espanso_engine::event::input::Key::F6 => espanso_inject::keys::Key::F6,
    espanso_engine::event::input::Key::F7 => espanso_inject::keys::Key::F7,
    espanso_engine::event::input::Key::F8 => espanso_inject::keys::Key::F8,
    espanso_engine::event::input::Key::F9 => espanso_inject::keys::Key::F9,
    espanso_engine::event::input::Key::F10 => espanso_inject::keys::Key::F10,
    espanso_engine::event::input::Key::F11 => espanso_inject::keys::Key::F11,
    espanso_engine::event::input::Key::F12 => espanso_inject::keys::Key::F12,
    espanso_engine::event::input::Key::F13 => espanso_inject::keys::Key::F13,
    espanso_engine::event::input::Key::F14 => espanso_inject::keys::Key::F14,
    espanso_engine::event::input::Key::F15 => espanso_inject::keys::Key::F15,
    espanso_engine::event::input::Key::F16 => espanso_inject::keys::Key::F16,
    espanso_engine::event::input::Key::F17 => espanso_inject::keys::Key::F17,
    espanso_engine::event::input::Key::F18 => espanso_inject::keys::Key::F18,
    espanso_engine::event::input::Key::F19 => espanso_inject::keys::Key::F19,
    espanso_engine::event::input::Key::F20 => espanso_inject::keys::Key::F20,
    espanso_engine::event::input::Key::Other(raw) => espanso_inject::keys::Key::Raw(*raw),
  }
}

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

use espanso_inject::Injector;

use crate::engine::dispatch::KeyInjector;

pub struct KeyInjectorAdapter<'a> {
  injector: &'a dyn Injector,
}

impl<'a> KeyInjectorAdapter<'a> {
  pub fn new(injector: &'a dyn Injector) -> Self {
    Self { injector }
  }
}

impl<'a> KeyInjector for KeyInjectorAdapter<'a> {
  fn inject_sequence(&self, keys: &[crate::engine::event::keyboard::Key]) -> anyhow::Result<()> {
    let converted_keys: Vec<_> = keys.iter().map(convert_to_inject_key).collect();
    self.injector.send_keys(&converted_keys, Default::default()) // TODO: handle options
  }
}

fn convert_to_inject_key(key: &crate::engine::event::keyboard::Key) -> espanso_inject::keys::Key {
  match key {
    crate::engine::event::keyboard::Key::Alt => espanso_inject::keys::Key::Alt,
    crate::engine::event::keyboard::Key::CapsLock => espanso_inject::keys::Key::CapsLock,
    crate::engine::event::keyboard::Key::Control => espanso_inject::keys::Key::Control,
    crate::engine::event::keyboard::Key::Meta => espanso_inject::keys::Key::Meta,
    crate::engine::event::keyboard::Key::NumLock => espanso_inject::keys::Key::NumLock,
    crate::engine::event::keyboard::Key::Shift => espanso_inject::keys::Key::Shift,
    crate::engine::event::keyboard::Key::Enter => espanso_inject::keys::Key::Enter,
    crate::engine::event::keyboard::Key::Tab => espanso_inject::keys::Key::Tab,
    crate::engine::event::keyboard::Key::Space => espanso_inject::keys::Key::Space,
    crate::engine::event::keyboard::Key::ArrowDown => espanso_inject::keys::Key::ArrowDown,
    crate::engine::event::keyboard::Key::ArrowLeft => espanso_inject::keys::Key::ArrowLeft,
    crate::engine::event::keyboard::Key::ArrowRight => espanso_inject::keys::Key::ArrowRight,
    crate::engine::event::keyboard::Key::ArrowUp => espanso_inject::keys::Key::ArrowUp,
    crate::engine::event::keyboard::Key::End => espanso_inject::keys::Key::End,
    crate::engine::event::keyboard::Key::Home => espanso_inject::keys::Key::Home,
    crate::engine::event::keyboard::Key::PageDown => espanso_inject::keys::Key::PageDown,
    crate::engine::event::keyboard::Key::PageUp => espanso_inject::keys::Key::PageUp,
    crate::engine::event::keyboard::Key::Escape => espanso_inject::keys::Key::Escape,
    crate::engine::event::keyboard::Key::Backspace => espanso_inject::keys::Key::Backspace,
    crate::engine::event::keyboard::Key::F1 => espanso_inject::keys::Key::F1,
    crate::engine::event::keyboard::Key::F2 => espanso_inject::keys::Key::F2,
    crate::engine::event::keyboard::Key::F3 => espanso_inject::keys::Key::F3,
    crate::engine::event::keyboard::Key::F4 => espanso_inject::keys::Key::F4,
    crate::engine::event::keyboard::Key::F5 => espanso_inject::keys::Key::F5,
    crate::engine::event::keyboard::Key::F6 => espanso_inject::keys::Key::F6,
    crate::engine::event::keyboard::Key::F7 => espanso_inject::keys::Key::F7,
    crate::engine::event::keyboard::Key::F8 => espanso_inject::keys::Key::F8,
    crate::engine::event::keyboard::Key::F9 => espanso_inject::keys::Key::F9,
    crate::engine::event::keyboard::Key::F10 => espanso_inject::keys::Key::F10,
    crate::engine::event::keyboard::Key::F11 => espanso_inject::keys::Key::F11,
    crate::engine::event::keyboard::Key::F12 => espanso_inject::keys::Key::F12,
    crate::engine::event::keyboard::Key::F13 => espanso_inject::keys::Key::F13,
    crate::engine::event::keyboard::Key::F14 => espanso_inject::keys::Key::F14,
    crate::engine::event::keyboard::Key::F15 => espanso_inject::keys::Key::F15,
    crate::engine::event::keyboard::Key::F16 => espanso_inject::keys::Key::F16,
    crate::engine::event::keyboard::Key::F17 => espanso_inject::keys::Key::F17,
    crate::engine::event::keyboard::Key::F18 => espanso_inject::keys::Key::F18,
    crate::engine::event::keyboard::Key::F19 => espanso_inject::keys::Key::F19,
    crate::engine::event::keyboard::Key::F20 => espanso_inject::keys::Key::F20,
    crate::engine::event::keyboard::Key::Other(raw) => espanso_inject::keys::Key::Raw(*raw),
  }
}

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

use crate::keys::Key;
use anyhow::Result;
use thiserror::Error;

pub fn convert_key_to_sym(key: &Key) -> Option<u32> {
  match key {
    Key::Alt => Some(0xFFE9),
    Key::CapsLock => Some(0xFFE5),
    Key::Control => Some(0xFFE3),
    Key::Meta => Some(0xFFEB),
    Key::NumLock => Some(0xFF7F),
    Key::Shift => Some(0xFFE1),

    // Whitespace
    Key::Enter => Some(0xFF0D),
    Key::Tab => Some(0xFF09),
    Key::Space => Some(0x20),

    // Navigation
    Key::ArrowDown => Some(0xFF54),
    Key::ArrowLeft => Some(0xFF51),
    Key::ArrowRight => Some(0xFF53),
    Key::ArrowUp => Some(0xFF52),
    Key::End => Some(0xFF57),
    Key::Home => Some(0xFF50),
    Key::PageDown => Some(0xFF56),
    Key::PageUp => Some(0xFF55),

    // UI keys
    Key::Escape => Some(0xFF1B),

    // Editing keys
    Key::Backspace => Some(0xFF08),
    Key::Insert => Some(0xff63),
    Key::Delete => Some(0xffff),

    // Function keys
    Key::F1 => Some(0xFFBE),
    Key::F2 => Some(0xFFBF),
    Key::F3 => Some(0xFFC0),
    Key::F4 => Some(0xFFC1),
    Key::F5 => Some(0xFFC2),
    Key::F6 => Some(0xFFC3),
    Key::F7 => Some(0xFFC4),
    Key::F8 => Some(0xFFC5),
    Key::F9 => Some(0xFFC6),
    Key::F10 => Some(0xFFC7),
    Key::F11 => Some(0xFFC8),
    Key::F12 => Some(0xFFC9),
    Key::F13 => Some(0xFFCA),
    Key::F14 => Some(0xFFCB),
    Key::F15 => Some(0xFFCC),
    Key::F16 => Some(0xFFCD),
    Key::F17 => Some(0xFFCE),
    Key::F18 => Some(0xFFCF),
    Key::F19 => Some(0xFFD0),
    Key::F20 => Some(0xFFD1),

    Key::A => Some(0x0061),
    Key::B => Some(0x0062),
    Key::C => Some(0x0063),
    Key::D => Some(0x0064),
    Key::E => Some(0x0065),
    Key::F => Some(0x0066),
    Key::G => Some(0x0067),
    Key::H => Some(0x0068),
    Key::I => Some(0x0069),
    Key::J => Some(0x006a),
    Key::K => Some(0x006b),
    Key::L => Some(0x006c),
    Key::M => Some(0x006d),
    Key::N => Some(0x006e),
    Key::O => Some(0x006f),
    Key::P => Some(0x0070),
    Key::Q => Some(0x0071),
    Key::R => Some(0x0072),
    Key::S => Some(0x0073),
    Key::T => Some(0x0074),
    Key::U => Some(0x0075),
    Key::V => Some(0x0076),
    Key::W => Some(0x0077),
    Key::X => Some(0x0078),
    Key::Y => Some(0x0079),
    Key::Z => Some(0x007a),

    Key::N0 => Some(0x0030),
    Key::N1 => Some(0x0031),
    Key::N2 => Some(0x0032),
    Key::N3 => Some(0x0033),
    Key::N4 => Some(0x0034),
    Key::N5 => Some(0x0035),
    Key::N6 => Some(0x0036),
    Key::N7 => Some(0x0037),
    Key::N8 => Some(0x0038),
    Key::N9 => Some(0x0039),
    Key::Numpad0 => Some(0xffb0),
    Key::Numpad1 => Some(0xffb1),
    Key::Numpad2 => Some(0xffb2),
    Key::Numpad3 => Some(0xffb3),
    Key::Numpad4 => Some(0xffb4),
    Key::Numpad5 => Some(0xffb5),
    Key::Numpad6 => Some(0xffb6),
    Key::Numpad7 => Some(0xffb7),
    Key::Numpad8 => Some(0xffb8),
    Key::Numpad9 => Some(0xffb9),
    Key::Raw(code) => Some(*code as u32),
  }
}

pub fn convert_to_sym_array(keys: &[Key]) -> Result<Vec<u64>> {
  let mut virtual_keys: Vec<u64> = Vec::new();
  for key in keys {
    let vk = convert_key_to_sym(key);
    if let Some(vk) = vk {
      virtual_keys.push(vk as u64);
    } else {
      return Err(LinuxRawKeyError::MappingFailure(key.clone()).into());
    }
  }
  Ok(virtual_keys)
}

#[derive(Error, Debug)]
pub enum LinuxRawKeyError {
  #[error("missing mapping for key `{0}`")]
  MappingFailure(Key),
}

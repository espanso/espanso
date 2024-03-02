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

pub fn convert_key_to_vkey(key: &Key) -> Option<i32> {
  match key {
    Key::Alt => Some(0x3A),
    Key::CapsLock => Some(0x39),
    Key::Control => Some(0x3B),
    Key::Meta => Some(0x37),
    Key::NumLock => None,
    Key::Shift => Some(0x38),
    Key::Enter => Some(0x24),
    Key::Tab => Some(0x30),
    Key::Space => Some(0x31),
    Key::ArrowDown => Some(0x7D),
    Key::ArrowLeft => Some(0x7B),
    Key::ArrowRight => Some(0x7C),
    Key::ArrowUp => Some(0x7E),
    Key::End => Some(0x77),
    Key::Home => Some(0x73),
    Key::PageDown => Some(0x79),
    Key::PageUp => Some(0x74),
    Key::Escape => Some(0x35),
    Key::Backspace => Some(0x33),
    Key::Insert => None,
    Key::Delete => Some(0x75),
    Key::F1 => Some(0x7A),
    Key::F2 => Some(0x78),
    Key::F3 => Some(0x63),
    Key::F4 => Some(0x76),
    Key::F5 => Some(0x60),
    Key::F6 => Some(0x61),
    Key::F7 => Some(0x62),
    Key::F8 => Some(0x64),
    Key::F9 => Some(0x65),
    Key::F10 => Some(0x6D),
    Key::F11 => Some(0x67),
    Key::F12 => Some(0x6F),
    Key::F13 => Some(0x69),
    Key::F14 => Some(0x6B),
    Key::F15 => Some(0x71),
    Key::F16 => Some(0x6A),
    Key::F17 => Some(0x40),
    Key::F18 => Some(0x4F),
    Key::F19 => Some(0x50),
    Key::F20 => Some(0x5A),
    Key::A => Some(0x00),
    Key::B => Some(0x0B),
    Key::C => Some(0x08),
    Key::D => Some(0x02),
    Key::E => Some(0x0E),
    Key::F => Some(0x03),
    Key::G => Some(0x05),
    Key::H => Some(0x04),
    Key::I => Some(0x22),
    Key::J => Some(0x26),
    Key::K => Some(0x28),
    Key::L => Some(0x25),
    Key::M => Some(0x2E),
    Key::N => Some(0x2D),
    Key::O => Some(0x1F),
    Key::P => Some(0x23),
    Key::Q => Some(0x0C),
    Key::R => Some(0x0F),
    Key::S => Some(0x01),
    Key::T => Some(0x11),
    Key::U => Some(0x20),
    Key::V => Some(0x09),
    Key::W => Some(0x0D),
    Key::X => Some(0x07),
    Key::Y => Some(0x10),
    Key::Z => Some(0x06),
    Key::N0 => Some(0x1D),
    Key::N1 => Some(0x12),
    Key::N2 => Some(0x13),
    Key::N3 => Some(0x14),
    Key::N4 => Some(0x15),
    Key::N5 => Some(0x17),
    Key::N6 => Some(0x16),
    Key::N7 => Some(0x1A),
    Key::N8 => Some(0x1C),
    Key::N9 => Some(0x19),
    Key::Numpad0 => Some(0x52),
    Key::Numpad1 => Some(0x53),
    Key::Numpad2 => Some(0x54),
    Key::Numpad3 => Some(0x55),
    Key::Numpad4 => Some(0x56),
    Key::Numpad5 => Some(0x57),
    Key::Numpad6 => Some(0x58),
    Key::Numpad7 => Some(0x59),
    Key::Numpad8 => Some(0x5B),
    Key::Numpad9 => Some(0x5C),
    Key::Raw(code) => Some(*code),
  }
}

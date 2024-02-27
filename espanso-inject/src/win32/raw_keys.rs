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
    let vkey = match key {
        Key::Alt => 0x12,
        Key::CapsLock => 0x14,
        Key::Control => 0x11,
        Key::Meta => 0x5B,
        Key::NumLock => 0x90,
        Key::Shift => 0xA0,
        Key::Enter => 0x0D,
        Key::Tab => 0x09,
        Key::Space => 0x20,
        Key::ArrowDown => 0x28,
        Key::ArrowLeft => 0x25,
        Key::ArrowRight => 0x27,
        Key::ArrowUp => 0x26,
        Key::End => 0x23,
        Key::Home => 0x24,
        Key::PageDown => 0x22,
        Key::PageUp => 0x21,
        Key::Escape => 0x1B,
        Key::Backspace => 0x08,
        Key::Insert => 0x2D,
        Key::Delete => 0x2E,
        Key::F1 => 0x70,
        Key::F2 => 0x71,
        Key::F3 => 0x72,
        Key::F4 => 0x73,
        Key::F5 => 0x74,
        Key::F6 => 0x75,
        Key::F7 => 0x76,
        Key::F8 => 0x77,
        Key::F9 => 0x78,
        Key::F10 => 0x79,
        Key::F11 => 0x7A,
        Key::F12 => 0x7B,
        Key::F13 => 0x7C,
        Key::F14 => 0x7D,
        Key::F15 => 0x7E,
        Key::F16 => 0x7F,
        Key::F17 => 0x80,
        Key::F18 => 0x81,
        Key::F19 => 0x82,
        Key::F20 => 0x83,
        Key::A => 0x41,
        Key::B => 0x42,
        Key::C => 0x43,
        Key::D => 0x44,
        Key::E => 0x45,
        Key::F => 0x46,
        Key::G => 0x47,
        Key::H => 0x48,
        Key::I => 0x49,
        Key::J => 0x4A,
        Key::K => 0x4B,
        Key::L => 0x4C,
        Key::M => 0x4D,
        Key::N => 0x4E,
        Key::O => 0x4F,
        Key::P => 0x50,
        Key::Q => 0x51,
        Key::R => 0x52,
        Key::S => 0x53,
        Key::T => 0x54,
        Key::U => 0x55,
        Key::V => 0x56,
        Key::W => 0x57,
        Key::X => 0x58,
        Key::Y => 0x59,
        Key::Z => 0x5A,
        Key::N0 => 0x30,
        Key::N1 => 0x31,
        Key::N2 => 0x32,
        Key::N3 => 0x33,
        Key::N4 => 0x34,
        Key::N5 => 0x35,
        Key::N6 => 0x36,
        Key::N7 => 0x37,
        Key::N8 => 0x38,
        Key::N9 => 0x39,
        Key::Numpad0 => 0x60,
        Key::Numpad1 => 0x61,
        Key::Numpad2 => 0x62,
        Key::Numpad3 => 0x63,
        Key::Numpad4 => 0x64,
        Key::Numpad5 => 0x65,
        Key::Numpad6 => 0x66,
        Key::Numpad7 => 0x67,
        Key::Numpad8 => 0x68,
        Key::Numpad9 => 0x69,
        Key::Raw(code) => *code,
    };
    Some(vkey)
}

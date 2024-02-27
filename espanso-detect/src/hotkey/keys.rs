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

use std::fmt::Display;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  static ref RAW_PARSER: Regex = Regex::new(r"^RAW\((\d+)\)$").unwrap();
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum ShortcutKey {
  Alt,
  Control,
  Meta,
  Shift,

  Enter,
  Tab,
  Space,
  Insert,

  // Navigation
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  End,
  Home,
  PageDown,
  PageUp,

  // Function ShortcutKeys
  F1,
  F2,
  F3,
  F4,
  F5,
  F6,
  F7,
  F8,
  F9,
  F10,
  F11,
  F12,
  F13,
  F14,
  F15,
  F16,
  F17,
  F18,
  F19,
  F20,

  // Alphabet
  A,
  B,
  C,
  D,
  E,
  F,
  G,
  H,
  I,
  J,
  K,
  L,
  M,
  N,
  O,
  P,
  Q,
  R,
  S,
  T,
  U,
  V,
  W,
  X,
  Y,
  Z,

  // Numbers
  N0,
  N1,
  N2,
  N3,
  N4,
  N5,
  N6,
  N7,
  N8,
  N9,

  // Numpad
  Numpad0,
  Numpad1,
  Numpad2,
  Numpad3,
  Numpad4,
  Numpad5,
  Numpad6,
  Numpad7,
  Numpad8,
  Numpad9,

  // Specify the raw platform-specific virtual key code.
  Raw(u32),
}

impl Display for ShortcutKey {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      ShortcutKey::Alt => write!(f, "ALT"),
      ShortcutKey::Control => write!(f, "CTRL"),
      ShortcutKey::Meta => write!(f, "META"),
      ShortcutKey::Shift => write!(f, "SHIFT"),
      ShortcutKey::Enter => write!(f, "ENTER"),
      ShortcutKey::Tab => write!(f, "TAB"),
      ShortcutKey::Space => write!(f, "SPACE"),
      ShortcutKey::Insert => write!(f, "INSERT"),
      ShortcutKey::ArrowDown => write!(f, "DOWN"),
      ShortcutKey::ArrowLeft => write!(f, "LEFT"),
      ShortcutKey::ArrowRight => write!(f, "RIGHT"),
      ShortcutKey::ArrowUp => write!(f, "UP"),
      ShortcutKey::End => write!(f, "END"),
      ShortcutKey::Home => write!(f, "HOME"),
      ShortcutKey::PageDown => write!(f, "PAGEDOWN"),
      ShortcutKey::PageUp => write!(f, "PAGEUP"),
      ShortcutKey::F1 => write!(f, "F1"),
      ShortcutKey::F2 => write!(f, "F2"),
      ShortcutKey::F3 => write!(f, "F3"),
      ShortcutKey::F4 => write!(f, "F4"),
      ShortcutKey::F5 => write!(f, "F5"),
      ShortcutKey::F6 => write!(f, "F6"),
      ShortcutKey::F7 => write!(f, "F7"),
      ShortcutKey::F8 => write!(f, "F8"),
      ShortcutKey::F9 => write!(f, "F9"),
      ShortcutKey::F10 => write!(f, "F10"),
      ShortcutKey::F11 => write!(f, "F11"),
      ShortcutKey::F12 => write!(f, "F12"),
      ShortcutKey::F13 => write!(f, "F13"),
      ShortcutKey::F14 => write!(f, "F14"),
      ShortcutKey::F15 => write!(f, "F15"),
      ShortcutKey::F16 => write!(f, "F16"),
      ShortcutKey::F17 => write!(f, "F17"),
      ShortcutKey::F18 => write!(f, "F18"),
      ShortcutKey::F19 => write!(f, "F19"),
      ShortcutKey::F20 => write!(f, "F20"),
      ShortcutKey::A => write!(f, "A"),
      ShortcutKey::B => write!(f, "B"),
      ShortcutKey::C => write!(f, "C"),
      ShortcutKey::D => write!(f, "D"),
      ShortcutKey::E => write!(f, "E"),
      ShortcutKey::F => write!(f, "F"),
      ShortcutKey::G => write!(f, "G"),
      ShortcutKey::H => write!(f, "H"),
      ShortcutKey::I => write!(f, "I"),
      ShortcutKey::J => write!(f, "J"),
      ShortcutKey::K => write!(f, "K"),
      ShortcutKey::L => write!(f, "L"),
      ShortcutKey::M => write!(f, "M"),
      ShortcutKey::N => write!(f, "N"),
      ShortcutKey::O => write!(f, "O"),
      ShortcutKey::P => write!(f, "P"),
      ShortcutKey::Q => write!(f, "Q"),
      ShortcutKey::R => write!(f, "R"),
      ShortcutKey::S => write!(f, "S"),
      ShortcutKey::T => write!(f, "T"),
      ShortcutKey::U => write!(f, "U"),
      ShortcutKey::V => write!(f, "V"),
      ShortcutKey::W => write!(f, "W"),
      ShortcutKey::X => write!(f, "X"),
      ShortcutKey::Y => write!(f, "Y"),
      ShortcutKey::Z => write!(f, "Z"),
      ShortcutKey::N0 => write!(f, "0"),
      ShortcutKey::N1 => write!(f, "1"),
      ShortcutKey::N2 => write!(f, "2"),
      ShortcutKey::N3 => write!(f, "3"),
      ShortcutKey::N4 => write!(f, "4"),
      ShortcutKey::N5 => write!(f, "5"),
      ShortcutKey::N6 => write!(f, "6"),
      ShortcutKey::N7 => write!(f, "7"),
      ShortcutKey::N8 => write!(f, "8"),
      ShortcutKey::N9 => write!(f, "9"),
      ShortcutKey::Numpad0 => write!(f, "NUMPAD0"),
      ShortcutKey::Numpad1 => write!(f, "NUMPAD1"),
      ShortcutKey::Numpad2 => write!(f, "NUMPAD2"),
      ShortcutKey::Numpad3 => write!(f, "NUMPAD3"),
      ShortcutKey::Numpad4 => write!(f, "NUMPAD4"),
      ShortcutKey::Numpad5 => write!(f, "NUMPAD5"),
      ShortcutKey::Numpad6 => write!(f, "NUMPAD6"),
      ShortcutKey::Numpad7 => write!(f, "NUMPAD7"),
      ShortcutKey::Numpad8 => write!(f, "NUMPAD8"),
      ShortcutKey::Numpad9 => write!(f, "NUMPAD9"),
      ShortcutKey::Raw(code) => write!(f, "RAW({code})"),
    }
  }
}

impl ShortcutKey {
  pub fn parse(key: &str) -> Option<ShortcutKey> {
    let parsed = match key {
      "ALT" | "OPTION" => Some(ShortcutKey::Alt),
      "CTRL" => Some(ShortcutKey::Control),
      "META" | "CMD" => Some(ShortcutKey::Meta),
      "SHIFT" => Some(ShortcutKey::Shift),
      "ENTER" => Some(ShortcutKey::Enter),
      "TAB" => Some(ShortcutKey::Tab),
      "SPACE" => Some(ShortcutKey::Space),
      "INSERT" => Some(ShortcutKey::Insert),
      "DOWN" => Some(ShortcutKey::ArrowDown),
      "LEFT" => Some(ShortcutKey::ArrowLeft),
      "RIGHT" => Some(ShortcutKey::ArrowRight),
      "UP" => Some(ShortcutKey::ArrowUp),
      "END" => Some(ShortcutKey::End),
      "HOME" => Some(ShortcutKey::Home),
      "PAGEDOWN" => Some(ShortcutKey::PageDown),
      "PAGEUP" => Some(ShortcutKey::PageUp),
      "F1" => Some(ShortcutKey::F1),
      "F2" => Some(ShortcutKey::F2),
      "F3" => Some(ShortcutKey::F3),
      "F4" => Some(ShortcutKey::F4),
      "F5" => Some(ShortcutKey::F5),
      "F6" => Some(ShortcutKey::F6),
      "F7" => Some(ShortcutKey::F7),
      "F8" => Some(ShortcutKey::F8),
      "F9" => Some(ShortcutKey::F9),
      "F10" => Some(ShortcutKey::F10),
      "F11" => Some(ShortcutKey::F11),
      "F12" => Some(ShortcutKey::F12),
      "F13" => Some(ShortcutKey::F13),
      "F14" => Some(ShortcutKey::F14),
      "F15" => Some(ShortcutKey::F15),
      "F16" => Some(ShortcutKey::F16),
      "F17" => Some(ShortcutKey::F17),
      "F18" => Some(ShortcutKey::F18),
      "F19" => Some(ShortcutKey::F19),
      "F20" => Some(ShortcutKey::F20),
      "A" => Some(ShortcutKey::A),
      "B" => Some(ShortcutKey::B),
      "C" => Some(ShortcutKey::C),
      "D" => Some(ShortcutKey::D),
      "E" => Some(ShortcutKey::E),
      "F" => Some(ShortcutKey::F),
      "G" => Some(ShortcutKey::G),
      "H" => Some(ShortcutKey::H),
      "I" => Some(ShortcutKey::I),
      "J" => Some(ShortcutKey::J),
      "K" => Some(ShortcutKey::K),
      "L" => Some(ShortcutKey::L),
      "M" => Some(ShortcutKey::M),
      "N" => Some(ShortcutKey::N),
      "O" => Some(ShortcutKey::O),
      "P" => Some(ShortcutKey::P),
      "Q" => Some(ShortcutKey::Q),
      "R" => Some(ShortcutKey::R),
      "S" => Some(ShortcutKey::S),
      "T" => Some(ShortcutKey::T),
      "U" => Some(ShortcutKey::U),
      "V" => Some(ShortcutKey::V),
      "W" => Some(ShortcutKey::W),
      "X" => Some(ShortcutKey::X),
      "Y" => Some(ShortcutKey::Y),
      "Z" => Some(ShortcutKey::Z),
      "0" => Some(ShortcutKey::N0),
      "1" => Some(ShortcutKey::N1),
      "2" => Some(ShortcutKey::N2),
      "3" => Some(ShortcutKey::N3),
      "4" => Some(ShortcutKey::N4),
      "5" => Some(ShortcutKey::N5),
      "6" => Some(ShortcutKey::N6),
      "7" => Some(ShortcutKey::N7),
      "8" => Some(ShortcutKey::N8),
      "9" => Some(ShortcutKey::N9),
      "NUMPAD0" => Some(ShortcutKey::Numpad0),
      "NUMPAD1" => Some(ShortcutKey::Numpad1),
      "NUMPAD2" => Some(ShortcutKey::Numpad2),
      "NUMPAD3" => Some(ShortcutKey::Numpad3),
      "NUMPAD4" => Some(ShortcutKey::Numpad4),
      "NUMPAD5" => Some(ShortcutKey::Numpad5),
      "NUMPAD6" => Some(ShortcutKey::Numpad6),
      "NUMPAD7" => Some(ShortcutKey::Numpad7),
      "NUMPAD8" => Some(ShortcutKey::Numpad8),
      "NUMPAD9" => Some(ShortcutKey::Numpad9),
      _ => None,
    };

    if parsed.is_none() {
      // Attempt to parse raw ShortcutKeys
      if RAW_PARSER.is_match(key) {
        if let Some(caps) = RAW_PARSER.captures(key) {
          let code_str = caps.get(1).map_or("", |m| m.as_str());
          let code = code_str.parse::<u32>();
          if let Ok(code) = code {
            return Some(ShortcutKey::Raw(code));
          }
        }
      }
    }

    parsed
  }

  // macOS keycodes

  #[cfg(target_os = "macos")]
  pub fn to_code(&self) -> Option<u32> {
    match self {
      ShortcutKey::Alt => Some(0x3A),
      ShortcutKey::Control => Some(0x3B),
      ShortcutKey::Meta => Some(0x37),
      ShortcutKey::Shift => Some(0x38),
      ShortcutKey::Enter => Some(0x24),
      ShortcutKey::Tab => Some(0x30),
      ShortcutKey::Space => Some(0x31),
      ShortcutKey::ArrowDown => Some(0x7D),
      ShortcutKey::ArrowLeft => Some(0x7B),
      ShortcutKey::ArrowRight => Some(0x7C),
      ShortcutKey::ArrowUp => Some(0x7E),
      ShortcutKey::End => Some(0x77),
      ShortcutKey::Home => Some(0x73),
      ShortcutKey::PageDown => Some(0x79),
      ShortcutKey::PageUp => Some(0x74),
      ShortcutKey::Insert => None,
      ShortcutKey::F1 => Some(0x7A),
      ShortcutKey::F2 => Some(0x78),
      ShortcutKey::F3 => Some(0x63),
      ShortcutKey::F4 => Some(0x76),
      ShortcutKey::F5 => Some(0x60),
      ShortcutKey::F6 => Some(0x61),
      ShortcutKey::F7 => Some(0x62),
      ShortcutKey::F8 => Some(0x64),
      ShortcutKey::F9 => Some(0x65),
      ShortcutKey::F10 => Some(0x6D),
      ShortcutKey::F11 => Some(0x67),
      ShortcutKey::F12 => Some(0x6F),
      ShortcutKey::F13 => Some(0x69),
      ShortcutKey::F14 => Some(0x6B),
      ShortcutKey::F15 => Some(0x71),
      ShortcutKey::F16 => Some(0x6A),
      ShortcutKey::F17 => Some(0x40),
      ShortcutKey::F18 => Some(0x4F),
      ShortcutKey::F19 => Some(0x50),
      ShortcutKey::F20 => Some(0x5A),
      ShortcutKey::A => Some(0x00),
      ShortcutKey::B => Some(0x0B),
      ShortcutKey::C => Some(0x08),
      ShortcutKey::D => Some(0x02),
      ShortcutKey::E => Some(0x0E),
      ShortcutKey::F => Some(0x03),
      ShortcutKey::G => Some(0x05),
      ShortcutKey::H => Some(0x04),
      ShortcutKey::I => Some(0x22),
      ShortcutKey::J => Some(0x26),
      ShortcutKey::K => Some(0x28),
      ShortcutKey::L => Some(0x25),
      ShortcutKey::M => Some(0x2E),
      ShortcutKey::N => Some(0x2D),
      ShortcutKey::O => Some(0x1F),
      ShortcutKey::P => Some(0x23),
      ShortcutKey::Q => Some(0x0C),
      ShortcutKey::R => Some(0x0F),
      ShortcutKey::S => Some(0x01),
      ShortcutKey::T => Some(0x11),
      ShortcutKey::U => Some(0x20),
      ShortcutKey::V => Some(0x09),
      ShortcutKey::W => Some(0x0D),
      ShortcutKey::X => Some(0x07),
      ShortcutKey::Y => Some(0x10),
      ShortcutKey::Z => Some(0x06),
      ShortcutKey::N0 => Some(0x1D),
      ShortcutKey::N1 => Some(0x12),
      ShortcutKey::N2 => Some(0x13),
      ShortcutKey::N3 => Some(0x14),
      ShortcutKey::N4 => Some(0x15),
      ShortcutKey::N5 => Some(0x17),
      ShortcutKey::N6 => Some(0x16),
      ShortcutKey::N7 => Some(0x1A),
      ShortcutKey::N8 => Some(0x1C),
      ShortcutKey::N9 => Some(0x19),
      ShortcutKey::Numpad0 => Some(0x52),
      ShortcutKey::Numpad1 => Some(0x53),
      ShortcutKey::Numpad2 => Some(0x54),
      ShortcutKey::Numpad3 => Some(0x55),
      ShortcutKey::Numpad4 => Some(0x56),
      ShortcutKey::Numpad5 => Some(0x57),
      ShortcutKey::Numpad6 => Some(0x58),
      ShortcutKey::Numpad7 => Some(0x59),
      ShortcutKey::Numpad8 => Some(0x5B),
      ShortcutKey::Numpad9 => Some(0x5C),
      ShortcutKey::Raw(code) => Some(*code),
    }
  }

  // Windows key codes

  #[cfg(target_os = "windows")]
  pub fn to_code(&self) -> Option<u32> {
    let vkey = match self {
      ShortcutKey::Alt => 0x12,
      ShortcutKey::Control => 0x11,
      ShortcutKey::Meta => 0x5B,
      ShortcutKey::Shift => 0xA0,
      ShortcutKey::Enter => 0x0D,
      ShortcutKey::Tab => 0x09,
      ShortcutKey::Space => 0x20,
      ShortcutKey::ArrowDown => 0x28,
      ShortcutKey::ArrowLeft => 0x25,
      ShortcutKey::ArrowRight => 0x27,
      ShortcutKey::ArrowUp => 0x26,
      ShortcutKey::End => 0x23,
      ShortcutKey::Home => 0x24,
      ShortcutKey::PageDown => 0x22,
      ShortcutKey::PageUp => 0x21,
      ShortcutKey::Insert => 0x2D,
      ShortcutKey::F1 => 0x70,
      ShortcutKey::F2 => 0x71,
      ShortcutKey::F3 => 0x72,
      ShortcutKey::F4 => 0x73,
      ShortcutKey::F5 => 0x74,
      ShortcutKey::F6 => 0x75,
      ShortcutKey::F7 => 0x76,
      ShortcutKey::F8 => 0x77,
      ShortcutKey::F9 => 0x78,
      ShortcutKey::F10 => 0x79,
      ShortcutKey::F11 => 0x7A,
      ShortcutKey::F12 => 0x7B,
      ShortcutKey::F13 => 0x7C,
      ShortcutKey::F14 => 0x7D,
      ShortcutKey::F15 => 0x7E,
      ShortcutKey::F16 => 0x7F,
      ShortcutKey::F17 => 0x80,
      ShortcutKey::F18 => 0x81,
      ShortcutKey::F19 => 0x82,
      ShortcutKey::F20 => 0x83,
      ShortcutKey::A => 0x41,
      ShortcutKey::B => 0x42,
      ShortcutKey::C => 0x43,
      ShortcutKey::D => 0x44,
      ShortcutKey::E => 0x45,
      ShortcutKey::F => 0x46,
      ShortcutKey::G => 0x47,
      ShortcutKey::H => 0x48,
      ShortcutKey::I => 0x49,
      ShortcutKey::J => 0x4A,
      ShortcutKey::K => 0x4B,
      ShortcutKey::L => 0x4C,
      ShortcutKey::M => 0x4D,
      ShortcutKey::N => 0x4E,
      ShortcutKey::O => 0x4F,
      ShortcutKey::P => 0x50,
      ShortcutKey::Q => 0x51,
      ShortcutKey::R => 0x52,
      ShortcutKey::S => 0x53,
      ShortcutKey::T => 0x54,
      ShortcutKey::U => 0x55,
      ShortcutKey::V => 0x56,
      ShortcutKey::W => 0x57,
      ShortcutKey::X => 0x58,
      ShortcutKey::Y => 0x59,
      ShortcutKey::Z => 0x5A,
      ShortcutKey::N0 => 0x30,
      ShortcutKey::N1 => 0x31,
      ShortcutKey::N2 => 0x32,
      ShortcutKey::N3 => 0x33,
      ShortcutKey::N4 => 0x34,
      ShortcutKey::N5 => 0x35,
      ShortcutKey::N6 => 0x36,
      ShortcutKey::N7 => 0x37,
      ShortcutKey::N8 => 0x38,
      ShortcutKey::N9 => 0x39,
      ShortcutKey::Numpad0 => 0x60,
      ShortcutKey::Numpad1 => 0x61,
      ShortcutKey::Numpad2 => 0x62,
      ShortcutKey::Numpad3 => 0x63,
      ShortcutKey::Numpad4 => 0x64,
      ShortcutKey::Numpad5 => 0x65,
      ShortcutKey::Numpad6 => 0x66,
      ShortcutKey::Numpad7 => 0x67,
      ShortcutKey::Numpad8 => 0x68,
      ShortcutKey::Numpad9 => 0x69,
      ShortcutKey::Raw(code) => *code,
    };
    Some(vkey)
  }

  // Linux mappings
  // NOTE: on linux, this method returns the KeySym and not the KeyCode
  //       which should be obtained in other ways depending on the backend.
  //       (X11 or Wayland)
  #[cfg(target_os = "linux")]
  pub fn to_code(&self) -> Option<u32> {
    match self {
      ShortcutKey::Alt => Some(0xFFE9),
      ShortcutKey::Control => Some(0xFFE3),
      ShortcutKey::Meta => Some(0xFFEB),
      ShortcutKey::Shift => Some(0xFFE1),
      ShortcutKey::Enter => Some(0xFF0D),
      ShortcutKey::Tab => Some(0xFF09),
      ShortcutKey::Space => Some(0x20),
      ShortcutKey::ArrowDown => Some(0xFF54),
      ShortcutKey::ArrowLeft => Some(0xFF51),
      ShortcutKey::ArrowRight => Some(0xFF53),
      ShortcutKey::ArrowUp => Some(0xFF52),
      ShortcutKey::End => Some(0xFF57),
      ShortcutKey::Home => Some(0xFF50),
      ShortcutKey::PageDown => Some(0xFF56),
      ShortcutKey::PageUp => Some(0xFF55),
      ShortcutKey::Insert => Some(0xff63),
      ShortcutKey::F1 => Some(0xFFBE),
      ShortcutKey::F2 => Some(0xFFBF),
      ShortcutKey::F3 => Some(0xFFC0),
      ShortcutKey::F4 => Some(0xFFC1),
      ShortcutKey::F5 => Some(0xFFC2),
      ShortcutKey::F6 => Some(0xFFC3),
      ShortcutKey::F7 => Some(0xFFC4),
      ShortcutKey::F8 => Some(0xFFC5),
      ShortcutKey::F9 => Some(0xFFC6),
      ShortcutKey::F10 => Some(0xFFC7),
      ShortcutKey::F11 => Some(0xFFC8),
      ShortcutKey::F12 => Some(0xFFC9),
      ShortcutKey::F13 => Some(0xFFCA),
      ShortcutKey::F14 => Some(0xFFCB),
      ShortcutKey::F15 => Some(0xFFCC),
      ShortcutKey::F16 => Some(0xFFCD),
      ShortcutKey::F17 => Some(0xFFCE),
      ShortcutKey::F18 => Some(0xFFCF),
      ShortcutKey::F19 => Some(0xFFD0),
      ShortcutKey::F20 => Some(0xFFD1),
      ShortcutKey::A => Some(0x0061),
      ShortcutKey::B => Some(0x0062),
      ShortcutKey::C => Some(0x0063),
      ShortcutKey::D => Some(0x0064),
      ShortcutKey::E => Some(0x0065),
      ShortcutKey::F => Some(0x0066),
      ShortcutKey::G => Some(0x0067),
      ShortcutKey::H => Some(0x0068),
      ShortcutKey::I => Some(0x0069),
      ShortcutKey::J => Some(0x006a),
      ShortcutKey::K => Some(0x006b),
      ShortcutKey::L => Some(0x006c),
      ShortcutKey::M => Some(0x006d),
      ShortcutKey::N => Some(0x006e),
      ShortcutKey::O => Some(0x006f),
      ShortcutKey::P => Some(0x0070),
      ShortcutKey::Q => Some(0x0071),
      ShortcutKey::R => Some(0x0072),
      ShortcutKey::S => Some(0x0073),
      ShortcutKey::T => Some(0x0074),
      ShortcutKey::U => Some(0x0075),
      ShortcutKey::V => Some(0x0076),
      ShortcutKey::W => Some(0x0077),
      ShortcutKey::X => Some(0x0078),
      ShortcutKey::Y => Some(0x0079),
      ShortcutKey::Z => Some(0x007a),
      ShortcutKey::N0 => Some(0x0030),
      ShortcutKey::N1 => Some(0x0031),
      ShortcutKey::N2 => Some(0x0032),
      ShortcutKey::N3 => Some(0x0033),
      ShortcutKey::N4 => Some(0x0034),
      ShortcutKey::N5 => Some(0x0035),
      ShortcutKey::N6 => Some(0x0036),
      ShortcutKey::N7 => Some(0x0037),
      ShortcutKey::N8 => Some(0x0038),
      ShortcutKey::N9 => Some(0x0039),
      ShortcutKey::Numpad0 => Some(0xffb0),
      ShortcutKey::Numpad1 => Some(0xffb1),
      ShortcutKey::Numpad2 => Some(0xffb2),
      ShortcutKey::Numpad3 => Some(0xffb3),
      ShortcutKey::Numpad4 => Some(0xffb4),
      ShortcutKey::Numpad5 => Some(0xffb5),
      ShortcutKey::Numpad6 => Some(0xffb6),
      ShortcutKey::Numpad7 => Some(0xffb7),
      ShortcutKey::Numpad8 => Some(0xffb8),
      ShortcutKey::Numpad9 => Some(0xffb9),
      ShortcutKey::Raw(code) => Some(*code),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_works_correctly() {
    assert!(matches!(
      ShortcutKey::parse("ALT").unwrap(),
      ShortcutKey::Alt
    ));
    assert!(matches!(
      ShortcutKey::parse("META").unwrap(),
      ShortcutKey::Meta
    ));
    assert!(matches!(
      ShortcutKey::parse("CMD").unwrap(),
      ShortcutKey::Meta
    ));
    assert!(matches!(
      ShortcutKey::parse("RAW(1234)").unwrap(),
      ShortcutKey::Raw(1234)
    ));
  }

  #[test]
  fn parse_invalid_keys() {
    assert!(ShortcutKey::parse("INVALID").is_none());
    assert!(ShortcutKey::parse("RAW(a)").is_none());
  }
}

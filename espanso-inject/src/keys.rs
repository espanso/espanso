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

#[derive(Debug, Clone)]
pub enum Key {
  // Modifiers
  Alt,
  CapsLock,
  Control,
  Meta,
  NumLock,
  Shift,

  // Whitespace
  Enter,
  Tab,
  Space,

  // Navigation
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  End,
  Home,
  PageDown,
  PageUp,

  // UI
  Escape,

  // Editing keys
  Backspace,
  Insert,
  Delete,

  // Function keys
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
  Raw(i32),
}

impl Display for Key {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      Key::Alt => write!(f, "ALT"),
      Key::CapsLock => write!(f, "CAPSLOCK"),
      Key::Control => write!(f, "CTRL"),
      Key::Meta => write!(f, "META"),
      Key::NumLock => write!(f, "NUMLOCK"),
      Key::Shift => write!(f, "SHIFT"),
      Key::Enter => write!(f, "ENTER"),
      Key::Tab => write!(f, "TAB"),
      Key::Space => write!(f, "SPACE"),
      Key::ArrowDown => write!(f, "DOWN"),
      Key::ArrowLeft => write!(f, "LEFT"),
      Key::ArrowRight => write!(f, "RIGHT"),
      Key::ArrowUp => write!(f, "UP"),
      Key::End => write!(f, "END"),
      Key::Home => write!(f, "HOME"),
      Key::PageDown => write!(f, "PAGEDOWN"),
      Key::PageUp => write!(f, "PAGEUP"),
      Key::Escape => write!(f, "ESC"),
      Key::Backspace => write!(f, "BACKSPACE"),
      Key::Insert => write!(f, "INSERT"),
      Key::Delete => write!(f, "DELETE"),
      Key::F1 => write!(f, "F1"),
      Key::F2 => write!(f, "F2"),
      Key::F3 => write!(f, "F3"),
      Key::F4 => write!(f, "F4"),
      Key::F5 => write!(f, "F5"),
      Key::F6 => write!(f, "F6"),
      Key::F7 => write!(f, "F7"),
      Key::F8 => write!(f, "F8"),
      Key::F9 => write!(f, "F9"),
      Key::F10 => write!(f, "F10"),
      Key::F11 => write!(f, "F11"),
      Key::F12 => write!(f, "F12"),
      Key::F13 => write!(f, "F13"),
      Key::F14 => write!(f, "F14"),
      Key::F15 => write!(f, "F15"),
      Key::F16 => write!(f, "F16"),
      Key::F17 => write!(f, "F17"),
      Key::F18 => write!(f, "F18"),
      Key::F19 => write!(f, "F19"),
      Key::F20 => write!(f, "F20"),
      Key::A => write!(f, "A"),
      Key::B => write!(f, "B"),
      Key::C => write!(f, "C"),
      Key::D => write!(f, "D"),
      Key::E => write!(f, "E"),
      Key::F => write!(f, "F"),
      Key::G => write!(f, "G"),
      Key::H => write!(f, "H"),
      Key::I => write!(f, "I"),
      Key::J => write!(f, "J"),
      Key::K => write!(f, "K"),
      Key::L => write!(f, "L"),
      Key::M => write!(f, "M"),
      Key::N => write!(f, "N"),
      Key::O => write!(f, "O"),
      Key::P => write!(f, "P"),
      Key::Q => write!(f, "Q"),
      Key::R => write!(f, "R"),
      Key::S => write!(f, "S"),
      Key::T => write!(f, "T"),
      Key::U => write!(f, "U"),
      Key::V => write!(f, "V"),
      Key::W => write!(f, "W"),
      Key::X => write!(f, "X"),
      Key::Y => write!(f, "Y"),
      Key::Z => write!(f, "Z"),
      Key::N0 => write!(f, "0"),
      Key::N1 => write!(f, "1"),
      Key::N2 => write!(f, "2"),
      Key::N3 => write!(f, "3"),
      Key::N4 => write!(f, "4"),
      Key::N5 => write!(f, "5"),
      Key::N6 => write!(f, "6"),
      Key::N7 => write!(f, "7"),
      Key::N8 => write!(f, "8"),
      Key::N9 => write!(f, "9"),
      Key::Numpad0 => write!(f, "NUMPAD0"),
      Key::Numpad1 => write!(f, "NUMPAD1"),
      Key::Numpad2 => write!(f, "NUMPAD2"),
      Key::Numpad3 => write!(f, "NUMPAD3"),
      Key::Numpad4 => write!(f, "NUMPAD4"),
      Key::Numpad5 => write!(f, "NUMPAD5"),
      Key::Numpad6 => write!(f, "NUMPAD6"),
      Key::Numpad7 => write!(f, "NUMPAD7"),
      Key::Numpad8 => write!(f, "NUMPAD8"),
      Key::Numpad9 => write!(f, "NUMPAD9"),
      Key::Raw(code) => write!(f, "RAW({code})"),
    }
  }
}

impl Key {
  pub fn parse(key: &str) -> Option<Key> {
    let parsed = match key {
      "ALT" | "OPTION" => Some(Key::Alt),
      "CAPSLOCK" => Some(Key::CapsLock),
      "CTRL" => Some(Key::Control),
      "META" | "CMD" => Some(Key::Meta),
      "NUMLOCK" => Some(Key::NumLock),
      "SHIFT" => Some(Key::Shift),
      "ENTER" => Some(Key::Enter),
      "TAB" => Some(Key::Tab),
      "SPACE" => Some(Key::Space),
      "DOWN" => Some(Key::ArrowDown),
      "LEFT" => Some(Key::ArrowLeft),
      "RIGHT" => Some(Key::ArrowRight),
      "UP" => Some(Key::ArrowUp),
      "END" => Some(Key::End),
      "HOME" => Some(Key::Home),
      "PAGEDOWN" => Some(Key::PageDown),
      "PAGEUP" => Some(Key::PageUp),
      "ESC" => Some(Key::Escape),
      "BACKSPACE" => Some(Key::Backspace),
      "INSERT" => Some(Key::Insert),
      "DELETE" => Some(Key::Delete),
      "F1" => Some(Key::F1),
      "F2" => Some(Key::F2),
      "F3" => Some(Key::F3),
      "F4" => Some(Key::F4),
      "F5" => Some(Key::F5),
      "F6" => Some(Key::F6),
      "F7" => Some(Key::F7),
      "F8" => Some(Key::F8),
      "F9" => Some(Key::F9),
      "F10" => Some(Key::F10),
      "F11" => Some(Key::F11),
      "F12" => Some(Key::F12),
      "F13" => Some(Key::F13),
      "F14" => Some(Key::F14),
      "F15" => Some(Key::F15),
      "F16" => Some(Key::F16),
      "F17" => Some(Key::F17),
      "F18" => Some(Key::F18),
      "F19" => Some(Key::F19),
      "F20" => Some(Key::F20),
      "A" => Some(Key::A),
      "B" => Some(Key::B),
      "C" => Some(Key::C),
      "D" => Some(Key::D),
      "E" => Some(Key::E),
      "F" => Some(Key::F),
      "G" => Some(Key::G),
      "H" => Some(Key::H),
      "I" => Some(Key::I),
      "J" => Some(Key::J),
      "K" => Some(Key::K),
      "L" => Some(Key::L),
      "M" => Some(Key::M),
      "N" => Some(Key::N),
      "O" => Some(Key::O),
      "P" => Some(Key::P),
      "Q" => Some(Key::Q),
      "R" => Some(Key::R),
      "S" => Some(Key::S),
      "T" => Some(Key::T),
      "U" => Some(Key::U),
      "V" => Some(Key::V),
      "W" => Some(Key::W),
      "X" => Some(Key::X),
      "Y" => Some(Key::Y),
      "Z" => Some(Key::Z),
      "0" => Some(Key::N0),
      "1" => Some(Key::N1),
      "2" => Some(Key::N2),
      "3" => Some(Key::N3),
      "4" => Some(Key::N4),
      "5" => Some(Key::N5),
      "6" => Some(Key::N6),
      "7" => Some(Key::N7),
      "8" => Some(Key::N8),
      "9" => Some(Key::N9),
      "NUMPAD0" => Some(Key::Numpad0),
      "NUMPAD1" => Some(Key::Numpad1),
      "NUMPAD2" => Some(Key::Numpad2),
      "NUMPAD3" => Some(Key::Numpad3),
      "NUMPAD4" => Some(Key::Numpad4),
      "NUMPAD5" => Some(Key::Numpad5),
      "NUMPAD6" => Some(Key::Numpad6),
      "NUMPAD7" => Some(Key::Numpad7),
      "NUMPAD8" => Some(Key::Numpad8),
      "NUMPAD9" => Some(Key::Numpad9),
      _ => None,
    };

    if parsed.is_none() {
      // Attempt to parse raw keys
      if RAW_PARSER.is_match(key) {
        if let Some(caps) = RAW_PARSER.captures(key) {
          let code_str = caps.get(1).map_or("", |m| m.as_str());
          let code = code_str.parse::<i32>();
          if let Ok(code) = code {
            return Some(Key::Raw(code));
          }
        }
      }
    }

    parsed
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_works_correctly() {
    assert!(matches!(Key::parse("ALT").unwrap(), Key::Alt));
    assert!(matches!(Key::parse("RAW(1234)").unwrap(), Key::Raw(1234)));
  }

  #[test]
  fn parse_invalid_keys() {
    assert!(Key::parse("INVALID").is_none());
    assert!(Key::parse("RAW(a)").is_none());
  }
}

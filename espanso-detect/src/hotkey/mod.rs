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

pub mod keys;

use std::fmt::Display;

use anyhow::Result;
use keys::ShortcutKey;
use thiserror::Error;

static MODIFIERS: &[ShortcutKey; 4] = &[
  ShortcutKey::Control,
  ShortcutKey::Alt,
  ShortcutKey::Shift,
  ShortcutKey::Meta,
];

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct HotKey {
  pub id: i32,
  pub key: ShortcutKey,
  pub modifiers: Vec<ShortcutKey>,
}

impl HotKey {
  pub fn new(id: i32, shortcut: &str) -> Result<Self> {
    let tokens: Vec<String> = shortcut
      .split('+')
      .map(|token| token.trim().to_uppercase())
      .collect();

    let mut modifiers = Vec::new();
    let mut main_key = None;
    for token in tokens {
      let key = ShortcutKey::parse(&token);
      match key {
        Some(key) => {
          if MODIFIERS.contains(&key) {
            modifiers.push(key)
          } else {
            main_key = Some(key)
          }
        }
        None => return Err(HotKeyError::InvalidKey(token).into()),
      };
    }

    if modifiers.is_empty() || main_key.is_none() {
      return Err(HotKeyError::InvalidShortcut(shortcut.to_string()).into());
    }

    Ok(Self {
      id,
      modifiers,
      key: main_key.unwrap(),
    })
  }

  #[allow(dead_code)]
  pub(crate) fn has_ctrl(&self) -> bool {
    self.modifiers.contains(&ShortcutKey::Control)
  }

  #[allow(dead_code)]
  pub(crate) fn has_meta(&self) -> bool {
    self.modifiers.contains(&ShortcutKey::Meta)
  }

  #[allow(dead_code)]
  pub(crate) fn has_alt(&self) -> bool {
    self.modifiers.contains(&ShortcutKey::Alt)
  }

  #[allow(dead_code)]
  pub(crate) fn has_shift(&self) -> bool {
    self.modifiers.contains(&ShortcutKey::Shift)
  }
}

impl Display for HotKey {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let str_modifiers: Vec<String> = self.modifiers.iter().map(|m| m.to_string()).collect();
    let modifiers = str_modifiers.join("+");
    write!(f, "{}+{}", &modifiers, &self.key)
  }
}

#[derive(Error, Debug)]
pub enum HotKeyError {
  #[error("invalid hotkey shortcut, `{0}` is not a valid key")]
  InvalidKey(String),

  #[error("invalid hotkey shortcut `{0}`")]
  InvalidShortcut(String),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_correctly() {
    assert_eq!(
      HotKey::new(1, "CTRL+V").unwrap(),
      HotKey {
        id: 1,
        key: ShortcutKey::V,
        modifiers: vec![ShortcutKey::Control],
      }
    );
    assert_eq!(
      HotKey::new(2, "SHIFT + Ctrl + v").unwrap(),
      HotKey {
        id: 2,
        key: ShortcutKey::V,
        modifiers: vec![ShortcutKey::Shift, ShortcutKey::Control],
      }
    );
    assert!(HotKey::new(3, "invalid").is_err());
  }

  #[test]
  fn modifiers_detected_correcty() {
    assert!(HotKey::new(1, "CTRL+V").unwrap().has_ctrl());
    assert!(HotKey::new(1, "ALT + V").unwrap().has_alt());
    assert!(HotKey::new(1, "CMD + V").unwrap().has_meta());
    assert!(HotKey::new(1, "SHIFT+ V").unwrap().has_shift());

    assert!(!HotKey::new(1, "SHIFT+ V").unwrap().has_ctrl());
    assert!(!HotKey::new(1, "SHIFT+ V").unwrap().has_alt());
    assert!(!HotKey::new(1, "SHIFT+ V").unwrap().has_meta());
  }
}

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

use log::error;

use crate::{
  event::{KeyboardEvent, Status},
  hotkey::HotKey,
};
use std::{collections::HashMap, time::Instant};

use super::state::State;

// Number of milliseconds that define how long the hotkey memory
// should retain pressed keys
const HOTKEY_WINDOW_TIMEOUT: u128 = 5000;

pub type KeySym = u32;
pub type KeyCode = u32;
pub type HotkeyMemoryMap = Vec<(KeyCode, Instant)>;

pub struct HotKeyFilter {
  map: HashMap<KeySym, KeyCode>,
  memory: HotkeyMemoryMap,
  hotkey_raw_map: HashMap<i32, Vec<KeyCode>>,
}

impl HotKeyFilter {
  pub fn new() -> Self {
    Self {
      map: HashMap::new(),
      memory: HotkeyMemoryMap::new(),
      hotkey_raw_map: HashMap::new(),
    }
  }

  pub fn initialize(&mut self, state: &State, hotkeys: &[HotKey]) {
    // First load the map
    self.map = HashMap::new();
    for code in 0..256 {
      if let Some(sym) = state.get_sym(code) {
        self.map.insert(sym, code);
      }
    }

    // Then the actual hotkeys
    self.hotkey_raw_map = hotkeys
      .iter()
      .filter_map(|hk| {
        let codes = Self::convert_hotkey_to_codes(self, hk);
        if codes.is_none() {
          error!("unable to register hotkey {:?}", hk);
        }
        Some((hk.id, codes?))
      })
      .collect();
  }

  pub fn process_event(&mut self, event: &KeyboardEvent) -> Option<i32> {
    let mut hotkey = None;
    let mut key_code = None;

    let mut to_be_removed = Vec::new();

    if event.status == Status::Released {
      // Remove from the memory all the key occurrences
      to_be_removed.extend(self.memory.iter().enumerate().filter_map(|(i, (code, _))| {
        if *code == event.code {
          Some(i)
        } else {
          None
        }
      }));
    } else {
      key_code = Some(event.code);
    }

    // Remove the old entries
    to_be_removed.extend(
      self
        .memory
        .iter()
        .enumerate()
        .filter_map(|(i, (_, instant))| {
          if instant.elapsed().as_millis() > HOTKEY_WINDOW_TIMEOUT {
            Some(i)
          } else {
            None
          }
        }),
    );

    // Remove duplicates and revert
    if !to_be_removed.is_empty() {
      #[allow(clippy::stable_sort_primitive)]
      to_be_removed.sort();
      to_be_removed.dedup();
      to_be_removed.reverse();
      for index in to_be_removed {
        self.memory.remove(index);
      }
    }

    if let Some(code) = key_code {
      self.memory.push((code, Instant::now()));

      for (id, codes) in &self.hotkey_raw_map {
        if codes
          .iter()
          .all(|hk_code| self.memory.iter().any(|(m_code, _)| m_code == hk_code))
        {
          hotkey = Some(*id);
          break;
        }
      }
    }

    hotkey
  }

  fn convert_hotkey_to_codes(&self, hk: &HotKey) -> Option<Vec<KeyCode>> {
    let mut codes = Vec::new();
    let key_code = self.map.get(&hk.key.to_code()?)?;
    codes.push(*key_code);

    for modifier in &hk.modifiers {
      let code = self.map.get(&modifier.to_code()?)?;
      codes.push(*code);
    }

    Some(codes)
  }
}

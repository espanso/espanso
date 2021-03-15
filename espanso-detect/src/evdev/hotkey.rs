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

use crate::{event::KeyboardEvent, hotkey::HotKey};
use std::{
  collections::{HashMap, VecDeque},
  time::Instant,
};

use super::state::State;

// Number of milliseconds between the first and last codes of an hotkey
// to be considered valid
const HOTKEY_WINDOW: i32 = 5000;

pub type KeySym = u32;
pub type KeyCode = u32;
pub type SymMap = HashMap<KeySym, KeyCode>;

pub type HotkeyMemoryMap = VecDeque<(KeyCode, Instant)>;

pub fn generate_sym_map(state: &State) -> HashMap<KeySym, KeyCode> {
  let mut map = HashMap::new();
  for code in 0..256 {
    if let Some(sym) = state.get_sym(code) {
      map.insert(sym, code);
    }
  }
  map
}

pub fn convert_hotkey_to_codes(hk: &HotKey, map: &SymMap) -> Option<Vec<KeyCode>> {
  let mut codes = Vec::new();
  let key_code = map.get(&hk.key.to_code()?)?;
  codes.push(*key_code);

  for modifier in hk.modifiers.iter() {
    let code = map.get(&modifier.to_code()?)?;
    codes.push(*code);
  }

  Some(codes)
}

pub fn detect_hotkey(
  event: &KeyboardEvent,
  memory: &mut HotkeyMemoryMap,
  hotkeys: &HashMap<i32, Vec<KeyCode>>,
) -> Option<i32> {
  // TODO: implement the actual matching mechanism
  // We need to "clean" the old entries
}

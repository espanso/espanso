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

mod context;
mod ffi;
mod keymap;
mod state;
mod uinput;

use std::{
  collections::{HashMap, HashSet},
  ffi::CString,
  time::Instant,
};

use context::Context;
use keymap::Keymap;
use log::{error, warn};
use uinput::UInputDevice;

use crate::{
  linux::raw_keys::convert_to_sym_array, InjectorCreationOptions, KeyboardStateProvider,
};
use anyhow::{bail, Result};
use itertools::Itertools;
use thiserror::Error;

use crate::{keys, InjectionOptions, Injector};

use self::state::State;

// Offset between evdev keycodes (where KEY_ESCAPE is 1), and the evdev XKB
// keycode set (where ESC is 9).
const EVDEV_OFFSET: u32 = 8;

// List of modifier keycodes, as defined in the "input-event-codes.h" header
// These can be overridden by changing the "evdev_modifier" option during initialization
const KEY_LEFTCTRL: u32 = 29;
const KEY_LEFTSHIFT: u32 = 42;
const KEY_RIGHTSHIFT: u32 = 54;
const KEY_LEFTALT: u32 = 56;
const KEY_LEFTMETA: u32 = 125;
const KEY_RIGHTMETA: u32 = 126;
const KEY_RIGHTCTRL: u32 = 97;
const KEY_RIGHTALT: u32 = 100;
const KEY_CAPSLOCK: u32 = 58;
const KEY_NUMLOCK: u32 = 69;

const DEFAULT_MODIFIERS: [u32; 10] = [
  KEY_LEFTCTRL,
  KEY_LEFTSHIFT,
  KEY_RIGHTSHIFT,
  KEY_LEFTALT,
  KEY_LEFTMETA,
  KEY_RIGHTMETA,
  KEY_RIGHTCTRL,
  KEY_RIGHTALT,
  KEY_CAPSLOCK,
  KEY_NUMLOCK,
];

const DEFAULT_MAX_MODIFIER_COMBINATION_LEN: i32 = 3;

// TODO: make the timeout a configurable option
const DEFAULT_WAIT_KEY_RELEASE_TIMEOUT_MS: u64 = 4000;

pub type KeySym = u32;

#[derive(Clone, Debug)]
struct KeyRecord {
  // Keycode
  code: u32,
  // List of modifiers that must be pressed
  modifiers: Vec<u32>,
}

type CharMap = HashMap<String, KeyRecord>;
type SymMap = HashMap<KeySym, KeyRecord>;

pub struct EVDEVInjector {
  device: UInputDevice,

  // Lookup maps
  char_map: CharMap,
  sym_map: SymMap,

  // Ownership
  _context: Context,
  _keymap: Keymap,

  // Keyboard state provider
  keyboard_state_provider: Option<Box<dyn KeyboardStateProvider>>,
}

#[allow(clippy::new_without_default)]
impl EVDEVInjector {
  pub fn new(options: InjectorCreationOptions) -> Result<Self> {
    let modifiers = options
      .evdev_modifiers
      .unwrap_or_else(|| DEFAULT_MODIFIERS.to_vec());
    let max_modifier_combination_len = options
      .evdev_max_modifier_combination_len
      .unwrap_or(DEFAULT_MAX_MODIFIER_COMBINATION_LEN);

    // Necessary to properly handle non-ascii chars
    let empty_string = CString::new("")?;
    unsafe {
      libc::setlocale(libc::LC_ALL, empty_string.as_ptr());
    }

    let context = Context::new().expect("unable to obtain xkb context");
    let keymap =
      Keymap::new(&context, options.evdev_keyboard_rmlvo).expect("unable to create xkb keymap");

    let (char_map, sym_map) =
      Self::generate_maps(&modifiers, max_modifier_combination_len, &keymap)?;

    // Create the uinput virtual device
    let device = UInputDevice::new()?;

    if options.keyboard_state_provider.is_none() {
      warn!("EVDEVInjection has been initialized without a KeyboardStateProvider, which might result in partial injections.");
    }

    Ok(Self {
      device,
      char_map,
      sym_map,
      _context: context,
      _keymap: keymap,
      keyboard_state_provider: options.keyboard_state_provider,
    })
  }

  fn generate_maps(
    modifiers: &[u32],
    max_modifier_sequence_len: i32,
    keymap: &Keymap,
  ) -> Result<(CharMap, SymMap)> {
    let mut char_map = HashMap::new();
    let mut sym_map = HashMap::new();

    let modifier_combinations = Self::generate_combinations(modifiers, max_modifier_sequence_len);

    // Cycle through all code/modifiers combinations to populate the reverse lookup tables
    for key_code in 8..256u32 {
      for modifier_combination in &modifier_combinations {
        let state = State::new(keymap)?;

        // Apply the modifiers
        for modifier in modifier_combination {
          // We need to add the EVDEV offset for xkbcommon to recognize it correctly
          state.update_key(*modifier + EVDEV_OFFSET, true);
        }

        let key_record = KeyRecord {
          code: key_code - EVDEV_OFFSET,
          modifiers: modifier_combination.clone(),
        };

        // Keysym was found
        if let Some(sym) = state.get_sym(key_code) {
          sym_map.entry(sym).or_insert_with(|| key_record.clone());
        }

        // Char was found
        if let Some(string) = state.get_string(key_code) {
          char_map.entry(string).or_insert(key_record);
        }
      }
    }

    Ok((char_map, sym_map))
  }

  fn generate_combinations(modifiers: &[u32], max_modifier_sequence_len: i32) -> Vec<Vec<u32>> {
    let mut combinations = vec![vec![]]; // Initial empty combination

    for sequence_len in 1..=max_modifier_sequence_len {
      let current_combinations = modifiers
        .iter()
        .copied()
        .combinations(sequence_len as usize);
      combinations.extend(current_combinations);
    }

    combinations
  }

  fn convert_to_record_array(&self, syms: &[u64]) -> Result<Vec<KeyRecord>> {
    syms
      .iter()
      .map(|sym| {
        self
          .sym_map
          .get(&(*sym as u32))
          .cloned()
          .ok_or_else(|| EVDEVInjectorError::SymMappingFailure(*sym as u32).into())
      })
      .collect()
  }

  fn send_key(&self, code: u32, pressed: bool, delay_us: u32) {
    self.device.emit(code, pressed);
    if delay_us != 0 {
      unsafe {
        libc::usleep(delay_us);
      }
    }
  }

  fn wait_until_key_is_released(&self, code: u32) -> Result<()> {
    if let Some(key_provider) = &self.keyboard_state_provider {
      let key_provider_code = code + EVDEV_OFFSET;

      if !key_provider.is_key_pressed(key_provider_code) {
        return Ok(());
      }

      // Key is pressed, wait until timeout
      let now = Instant::now();
      while now.elapsed() < std::time::Duration::from_millis(DEFAULT_WAIT_KEY_RELEASE_TIMEOUT_MS) {
        if !key_provider.is_key_pressed(key_provider_code) {
          return Ok(());
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
      }

      bail!("timed-out while waiting for key release: {}", code);
    }
    // else Keyboard provider not available,
    Ok(())
  }

  fn wait_delay(&self, delay_ms: u32) {
    if delay_ms > 0 {
      std::thread::sleep(std::time::Duration::from_millis(delay_ms as u64));
    }
  }
}

impl Injector for EVDEVInjector {
  fn send_string(&self, string: &str, options: InjectionOptions) -> Result<()> {
    // Compute all the key record sequence first to make sure a mapping is available
    let records: Result<Vec<KeyRecord>> = string
      .chars()
      .map(|c| c.to_string())
      .map(|char| {
        self
          .char_map
          .get(&char)
          .cloned()
          .ok_or_else(|| EVDEVInjectorError::CharMappingFailure(char).into())
      })
      .collect();

    let delay_us = options.delay as u32 * 1000; // Convert to micro seconds
    let modifier_delay_ms = options.evdev_modifier_delay;

    // We need to keep track of the modifiers currently pressed to
    // press or release them accordingly
    let mut current_modifiers: HashSet<u32> = HashSet::new();

    for record in records? {
      let record_modifiers = record.modifiers.iter().copied().collect::<HashSet<_>>();

      // Release all the modifiers that are not needed anymore
      for expired_modifier in current_modifiers.difference(&record_modifiers) {
        self.wait_delay(modifier_delay_ms);

        self.send_key(*expired_modifier, false, delay_us);

        self.wait_delay(modifier_delay_ms);
      }

      // Press all the new modifiers that are now needed
      for new_modifier in record_modifiers.difference(&current_modifiers) {
        self.wait_delay(modifier_delay_ms);

        self.wait_until_key_is_released(record.code)?;
        self.send_key(*new_modifier, true, delay_us);

        self.wait_delay(modifier_delay_ms);
      }

      // Send the char
      self.wait_until_key_is_released(record.code)?;
      self.send_key(record.code, true, delay_us);
      self.send_key(record.code, false, delay_us);

      current_modifiers = record_modifiers;
    }

    // Release all the remaining modifiers
    for expired_modifier in current_modifiers {
      self.send_key(expired_modifier, false, delay_us);
    }

    Ok(())
  }

  fn send_keys(&self, keys: &[keys::Key], options: InjectionOptions) -> Result<()> {
    // Compute all the key record sequence first to make sure a mapping is available
    let syms = convert_to_sym_array(keys)?;
    let records = self.convert_to_record_array(&syms)?;

    let delay_us = options.delay as u32 * 1000; // Convert to micro seconds

    for record in records {
      // Press the modifiers
      for modifier in &record.modifiers {
        self.send_key(*modifier, true, delay_us);
      }

      // Send the key
      self.send_key(record.code, true, delay_us);
      self.send_key(record.code, false, delay_us);

      // Release the modifiers
      for modifier in &record.modifiers {
        self.send_key(*modifier, false, delay_us);
      }
    }

    Ok(())
  }

  fn send_key_combination(&self, keys: &[keys::Key], options: InjectionOptions) -> Result<()> {
    // Compute all the key record sequence first to make sure a mapping is available
    let syms = convert_to_sym_array(keys)?;
    let records = self.convert_to_record_array(&syms)?;

    let delay_us = options.delay as u32 * 1000; // Convert to micro seconds

    // First press the keys
    for record in &records {
      // Press the modifiers
      for modifier in &record.modifiers {
        self.send_key(*modifier, true, delay_us);
      }

      // Send the key
      self.send_key(record.code, true, delay_us);
    }

    // Then release them
    for record in records.iter().rev() {
      self.send_key(record.code, false, delay_us);

      // Release the modifiers
      for modifier in &record.modifiers {
        self.send_key(*modifier, false, delay_us);
      }
    }

    Ok(())
  }
}

#[derive(Error, Debug)]
pub enum EVDEVInjectorError {
  #[error("missing vkey mapping for char `{0}`")]
  CharMappingFailure(String),

  #[error("missing record mapping for sym `{0}`")]
  SymMappingFailure(u32),
}

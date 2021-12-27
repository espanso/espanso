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

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use espanso_inject::KeyboardStateProvider;
use log::warn;

/// This duration represents the maximum length for which a pressed key
/// event is considered valid. This is useful when the "release" event is
/// lost for whatever reason, so that espanso becomes eventually consistent
/// after a while.
const KEY_PRESS_EVENT_INVALIDATION_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub struct KeyStateStore {
  state: Arc<Mutex<KeyState>>,
}

impl KeyStateStore {
  pub fn new() -> Self {
    Self {
      state: Arc::new(Mutex::new(KeyState::default())),
    }
  }

  pub fn is_key_pressed(&self, key_code: u32) -> bool {
    let mut state = self.state.lock().expect("unable to obtain modifier state");

    if let Some(status) = state.keys.get_mut(&key_code) {
      if status.is_outdated() {
        warn!(
          "detected outdated key records for {:?}, releasing the state",
          key_code
        );
        status.release();
      }

      status.is_pressed()
    } else {
      false
    }
  }

  pub fn update_state(&self, key_code: u32, is_pressed: bool) {
    let mut state = self.state.lock().expect("unable to obtain key state");
    if let Some(status) = state.keys.get_mut(&key_code) {
      if is_pressed {
        status.press();
      } else {
        status.release();
      }
    } else {
      state.keys.insert(key_code, KeyStatus::new(is_pressed));
    }
  }
}

#[derive(Default)]
struct KeyState {
  keys: HashMap<u32, KeyStatus>,
}

struct KeyStatus {
  pressed_at: Option<Instant>,
}

impl KeyStatus {
  fn new(is_pressed: bool) -> Self {
    Self {
      pressed_at: if is_pressed {
        Some(Instant::now())
      } else {
        None
      },
    }
  }

  fn is_pressed(&self) -> bool {
    self.pressed_at.is_some()
  }

  fn is_outdated(&self) -> bool {
    let now = Instant::now();
    if let Some(pressed_at) = self.pressed_at {
      now.duration_since(pressed_at) > KEY_PRESS_EVENT_INVALIDATION_TIMEOUT
    } else {
      false
    }
  }

  fn release(&mut self) {
    self.pressed_at = None
  }

  fn press(&mut self) {
    self.pressed_at = Some(Instant::now());
  }
}

impl KeyboardStateProvider for KeyStateStore {
  fn is_key_pressed(&self, code: u32) -> bool {
    self.is_key_pressed(code)
  }
}

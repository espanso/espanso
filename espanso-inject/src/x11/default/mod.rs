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
  collections::{HashMap, HashSet},
  ffi::{CStr, CString},
  os::raw::c_char,
  slice,
};

use crate::x11::ffi::{
  Display, KeyCode, KeyPress, KeyRelease, KeySym, Window, XCloseDisplay, XDefaultRootWindow,
  XFlush, XFreeModifiermap, XGetInputFocus, XGetModifierMapping, XKeyEvent, XQueryKeymap,
  XSendEvent, XSync, XTestFakeKeyEvent,
};
use libc::c_void;
use log::{debug, error};

use crate::{linux::raw_keys::convert_to_sym_array, x11::ffi::Xutf8LookupString};
use anyhow::{bail, Result};
use thiserror::Error;

use crate::{keys, InjectionOptions, Injector};

use crate::x11::ffi::{
  XCloseIM, XCreateIC, XDestroyIC, XFilterEvent, XFree, XIMPreeditNothing, XIMStatusNothing,
  XNClientWindow_0, XNInputStyle_0, XOpenIM, XmbResetIC, XIC,
};

// Offset between evdev keycodes (where KEY_ESCAPE is 1), and the evdev XKB
// keycode set (where ESC is 9).
const EVDEV_OFFSET: u32 = 8;

#[derive(Clone, Copy, Debug)]
struct KeyPair {
  // Keycode
  code: u32,
  // Modifier state which combined with the code produces the char
  // This is a bit mask:
  state: u32,
}

#[derive(Clone, Copy, Debug)]
struct KeyRecord {
  main: KeyPair,

  // Under some keyboard layouts (de, es), a deadkey
  // press might be needed to generate the right char
  preceding_dead_key: Option<KeyPair>,
}

type CharMap = HashMap<String, KeyRecord>;
type SymMap = HashMap<KeySym, KeyRecord>;

pub struct X11DefaultInjector {
  display: *mut Display,

  char_map: CharMap,
  sym_map: SymMap,
}

#[allow(clippy::new_without_default)]
impl X11DefaultInjector {
  pub fn new() -> Result<Self> {
    // Necessary to properly handle non-ascii chars
    let empty_string = CString::new("")?;
    unsafe {
      libc::setlocale(libc::LC_ALL, empty_string.as_ptr());
    }

    let display = unsafe { crate::x11::ffi::XOpenDisplay(std::ptr::null()) };
    if display.is_null() {
      return Err(X11InjectorError::Init().into());
    }

    let (char_map, sym_map) = Self::generate_maps(display)?;

    Ok(Self {
      display,
      char_map,
      sym_map,
    })
  }

  fn generate_maps(display: *mut Display) -> Result<(CharMap, SymMap)> {
    debug!("generating key maps");

    let mut char_map = HashMap::new();
    let mut sym_map = HashMap::new();

    let input_method = unsafe {
      XOpenIM(
        display,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
      )
    };
    if input_method.is_null() {
      bail!("could not open input method");
    }
    let _im_guard = scopeguard::guard((), |()| {
      unsafe { XCloseIM(input_method) };
    });

    let input_context = unsafe {
      XCreateIC(
        input_method,
        XNInputStyle_0.as_ptr(),
        XIMPreeditNothing | XIMStatusNothing,
        XNClientWindow_0.as_ptr(),
        0,
        std::ptr::null_mut(),
      )
    };
    if input_context.is_null() {
      bail!("could not open input context");
    }
    let _ic_guard = scopeguard::guard((), |()| {
      unsafe { XDestroyIC(input_context) };
    });

    let deadkeys = Self::find_deadkeys(display, &input_context)?;

    // Cycle through all state/code combinations to populate the reverse lookup tables
    for key_code in 0..256u32 {
      for modifier_state in 0..256u32 {
        for dead_key in &deadkeys {
          let code_with_offset = key_code + EVDEV_OFFSET;

          let preceding_dead_key = if let Some(dead_key) = dead_key {
            let mut dead_key_event = XKeyEvent {
              display,
              keycode: dead_key.code,
              state: dead_key.state,

              // These might not even need to be filled
              window: 0,
              root: 0,
              same_screen: 1,
              time: 0,
              type_: KeyPress,
              x_root: 1,
              y_root: 1,
              x: 1,
              y: 1,
              subwindow: 0,
              serial: 0,
              send_event: 0,
            };

            unsafe { XFilterEvent(&mut dead_key_event, 0) };

            Some(*dead_key)
          } else {
            None
          };

          let mut key_event = XKeyEvent {
            display,
            keycode: code_with_offset,
            state: modifier_state,

            // These might not even need to be filled
            window: 0,
            root: 0,
            same_screen: 1,
            time: 0,
            type_: KeyPress,
            x_root: 1,
            y_root: 1,
            x: 1,
            y: 1,
            subwindow: 0,
            serial: 0,
            send_event: 0,
          };

          unsafe { XFilterEvent(&mut key_event, 0) };
          let mut sym: KeySym = 0;
          let mut buffer: [c_char; 10] = [0; 10];

          let result = unsafe {
            Xutf8LookupString(
              input_context,
              &mut key_event,
              buffer.as_mut_ptr(),
              (buffer.len() - 1) as i32,
              &mut sym,
              std::ptr::null_mut(),
            )
          };

          let key_record = KeyRecord {
            main: KeyPair {
              code: code_with_offset,
              state: modifier_state,
            },
            preceding_dead_key,
          };

          // Keysym was found
          if sym != 0 {
            sym_map.entry(sym).or_insert(key_record);
          };

          // Char was found
          if result > 0 {
            let raw_string = unsafe { CStr::from_ptr(buffer.as_ptr()) };
            let string = raw_string.to_string_lossy().to_string();
            char_map.entry(string).or_insert(key_record);
          };

          // We need to reset the context state to prevent
          // deadkeys effect to propagate to the next combination
          let _reset = unsafe { XmbResetIC(input_context) };
          unsafe { XFree(_reset as *mut c_void) };
        }
      }
    }

    debug!("Populated char_map with {} symbols", char_map.len());
    debug!("Populated sym_map with {} symbols", sym_map.len());
    debug!("Detected {} dead key combinations", deadkeys.len());

    Ok((char_map, sym_map))
  }

  fn find_deadkeys(display: *mut Display, input_context: &XIC) -> Result<Vec<Option<KeyPair>>> {
    let mut deadkeys = vec![None];
    let mut seen_keysyms: HashSet<KeySym> = HashSet::new();

    // Cycle through all state/code combinations to populate the reverse lookup tables
    for key_code in 0..256u32 {
      for modifier_state in 0..256u32 {
        let code_with_offset = key_code + EVDEV_OFFSET;
        let mut event = XKeyEvent {
          display,
          keycode: code_with_offset,
          state: modifier_state,

          // These might not even need to be filled
          window: 0,
          root: 0,
          same_screen: 1,
          time: 0,
          type_: KeyPress,
          x_root: 1,
          y_root: 1,
          x: 1,
          y: 1,
          subwindow: 0,
          serial: 0,
          send_event: 0,
        };

        let filter = unsafe { XFilterEvent(&mut event, 0) };
        if filter == 1 {
          let mut sym: KeySym = 0;
          let mut buffer: [c_char; 10] = [0; 10];

          unsafe {
            Xutf8LookupString(
              *input_context,
              &mut event,
              buffer.as_mut_ptr(),
              (buffer.len() - 1) as i32,
              &mut sym,
              std::ptr::null_mut(),
            )
          };

          if sym != 0 && !seen_keysyms.contains(&sym) {
            let key_record = KeyPair {
              code: code_with_offset,
              state: modifier_state,
            };
            deadkeys.push(Some(key_record));
            seen_keysyms.insert(sym);
          }
        }

        let _reset = unsafe { XmbResetIC(*input_context) };
        unsafe { XFree(_reset as *mut c_void) };
      }
    }

    Ok(deadkeys)
  }

  fn convert_to_record_array(&self, syms: &[KeySym]) -> Result<Vec<KeyRecord>> {
    syms
      .iter()
      .map(|sym| {
        self
          .sym_map
          .get(sym)
          .copied()
          .ok_or_else(|| X11InjectorError::SymMapping(*sym).into())
      })
      .collect()
  }

  // This method was inspired by the wonderful xdotool by Jordan Sissel
  // https://github.com/jordansissel/xdotool
  fn get_modifier_codes(&self) -> Vec<Vec<KeyCode>> {
    let modifiers_ptr = unsafe { XGetModifierMapping(self.display) };
    let modifiers = unsafe { *modifiers_ptr };

    let mut modifiers_codes = Vec::new();

    for mod_index in 0..=7 {
      let mut modifier_codes = Vec::new();
      for mod_key in 0..modifiers.max_keypermod {
        let modifier_map = unsafe {
          slice::from_raw_parts(
            modifiers.modifiermap,
            (8 * modifiers.max_keypermod) as usize,
          )
        };
        let keycode = modifier_map[(mod_index * modifiers.max_keypermod + mod_key) as usize];
        if keycode != 0 {
          modifier_codes.push(keycode);
        }
      }
      modifiers_codes.push(modifier_codes);
    }

    unsafe { XFreeModifiermap(modifiers_ptr) };

    modifiers_codes
  }

  fn render_key_combination(&self, original_records: &[KeyRecord]) -> Vec<KeyRecord> {
    let modifiers_codes = self.get_modifier_codes();
    let mut records = Vec::new();

    let mut current_state = 0u32;
    for record in original_records {
      let mut current_record = *record;
      current_record.main.state = current_state;
      records.push(current_record);

      // Calculate the state for the next record by applying the modifiers
      for (mod_index, modifier) in modifiers_codes.iter().enumerate() {
        if modifier.contains(&(record.main.code as u8)) {
          current_state |= 1 << mod_index;
        }
      }
    }

    records
  }

  fn get_focused_window(&self) -> Window {
    let mut focused_window: Window = 0;
    let mut revert_to = 0;
    unsafe {
      XGetInputFocus(self.display, &mut focused_window, &mut revert_to);
    }
    focused_window
  }

  fn send_key(&self, window: Window, record: &KeyPair, pressed: bool, delay_us: u32) {
    let root_window = unsafe { XDefaultRootWindow(self.display) };
    let mut event = XKeyEvent {
      display: self.display,
      keycode: record.code,
      state: record.state,
      window,
      root: root_window,
      same_screen: 1,
      time: 0,
      type_: if pressed { KeyPress } else { KeyRelease },
      x_root: 1,
      y_root: 1,
      x: 1,
      y: 1,
      subwindow: 0,
      serial: 0,
      send_event: 0,
    };
    unsafe {
      XSendEvent(self.display, window, 1, 0, &mut event);
      XFlush(self.display);
    }

    if delay_us != 0 {
      unsafe {
        libc::usleep(delay_us);
      }
    }
  }

  fn xtest_send_modifiers(&self, modmask: u32, pressed: bool) {
    let modifiers_codes = self.get_modifier_codes();
    for (mod_index, modifier_codes) in modifiers_codes.into_iter().enumerate() {
      if (modmask & (1 << mod_index)) != 0 {
        for keycode in modifier_codes {
          let is_press = i32::from(pressed);
          unsafe {
            XTestFakeKeyEvent(self.display, keycode as u32, is_press, 0);
            XSync(self.display, 0);
          }
        }
      }
    }
  }

  fn xtest_send_key(&self, record: &KeyPair, pressed: bool, delay_us: u32) {
    // If the key requires any modifier, we need to send those events
    if record.state != 0 {
      self.xtest_send_modifiers(record.state, pressed);
    }

    let is_press = i32::from(pressed);
    unsafe {
      XTestFakeKeyEvent(self.display, record.code, is_press, 0);
      XSync(self.display, 0);
      XFlush(self.display);
    }

    if delay_us != 0 {
      unsafe {
        libc::usleep(delay_us);
      }
    }
  }

  fn xtest_release_all_keys(&self) {
    let mut keys: [u8; 32] = [0; 32];
    unsafe {
      XQueryKeymap(self.display, keys.as_mut_ptr());
    }

    #[allow(clippy::needless_range_loop)]
    for i in 0..32 {
      // Only those that are pressed should be changed
      if keys[i] != 0 {
        for k in 0..8 {
          if (keys[i] & (1 << k)) != 0 {
            let key_code = i * 8 + k;
            unsafe {
              XTestFakeKeyEvent(self.display, key_code as u32, 0, 0);
            }
          }
        }
      }
    }
  }
}

impl Drop for X11DefaultInjector {
  fn drop(&mut self) {
    unsafe {
      XCloseDisplay(self.display);
    }
  }
}

impl Injector for X11DefaultInjector {
  fn send_string(&self, string: &str, options: InjectionOptions) -> Result<()> {
    let focused_window = self.get_focused_window();

    if options.disable_fast_inject {
      self.xtest_release_all_keys();
    }

    // Compute all the key record sequence first to make sure a mapping is available
    let records: Result<Vec<KeyRecord>> = string
      .chars()
      .map(|c| c.to_string())
      .map(|char| {
        self
          .char_map
          .get(&char)
          .copied()
          .ok_or_else(|| X11InjectorError::CharMapping(char).into())
      })
      .collect();

    let delay_us = options.delay as u32 * 1000; // Convert to micro seconds

    for record in records? {
      if options.disable_fast_inject {
        if let Some(deadkey) = &record.preceding_dead_key {
          self.xtest_send_key(deadkey, true, delay_us);
          self.xtest_send_key(deadkey, false, delay_us);
        }

        self.xtest_send_key(&record.main, true, delay_us);
        self.xtest_send_key(&record.main, false, delay_us);
      } else {
        if let Some(deadkey) = &record.preceding_dead_key {
          self.send_key(focused_window, deadkey, true, delay_us);
          self.send_key(focused_window, deadkey, false, delay_us);
        }

        self.send_key(focused_window, &record.main, true, delay_us);
        self.send_key(focused_window, &record.main, false, delay_us);
      }
    }

    Ok(())
  }

  fn send_keys(&self, keys: &[keys::Key], options: InjectionOptions) -> Result<()> {
    let focused_window = self.get_focused_window();

    // Compute all the key record sequence first to make sure a mapping is available
    let syms = convert_to_sym_array(keys)?;
    let records = self.convert_to_record_array(&syms)?;

    if options.disable_fast_inject {
      self.xtest_release_all_keys();
    }

    let delay_us = options.delay as u32 * 1000; // Convert to micro seconds

    for record in records {
      if options.disable_fast_inject {
        self.xtest_send_key(&record.main, true, delay_us);
        self.xtest_send_key(&record.main, false, delay_us);
      } else {
        self.send_key(focused_window, &record.main, true, delay_us);
        self.send_key(focused_window, &record.main, false, delay_us);
      }
    }

    Ok(())
  }

  fn send_key_combination(&self, keys: &[keys::Key], options: InjectionOptions) -> Result<()> {
    let focused_window = self.get_focused_window();

    // Compute all the key record sequence first to make sure a mapping is available
    let syms = convert_to_sym_array(keys)?;
    let records = self.convert_to_record_array(&syms)?;

    // Render the correct modifier mask for the given sequence
    let records = self.render_key_combination(&records);

    if options.disable_fast_inject {
      self.xtest_release_all_keys();
    }

    let delay_us = options.delay as u32 * 1000; // Convert to micro seconds

    // First press the keys
    for record in &records {
      if options.disable_fast_inject {
        self.xtest_send_key(&record.main, true, delay_us);
      } else {
        self.send_key(focused_window, &record.main, true, delay_us);
      }
    }

    // Then release them
    for record in records.iter().rev() {
      if options.disable_fast_inject {
        self.xtest_send_key(&record.main, false, delay_us);
      } else {
        self.send_key(focused_window, &record.main, false, delay_us);
      }
    }

    Ok(())
  }
}

#[derive(Error, Debug)]
pub enum X11InjectorError {
  #[error("failed to initialize x11 display")]
  Init(),

  #[error("missing vkey mapping for char `{0}`")]
  CharMapping(String),

  #[error("missing record mapping for sym `{0}`")]
  SymMapping(u64),
}

// This code is a port of the libxkbcommon "interactive-evdev.c" example
// https://github.com/xkbcommon/libxkbcommon/blob/master/tools/interactive-evdev.c

use anyhow::Result;
use libc::{input_event, size_t, ssize_t, ENODEV, EWOULDBLOCK, O_CLOEXEC, O_NONBLOCK, O_RDONLY};
use log::trace;
use scopeguard::ScopeGuard;
use std::collections::HashMap;
use std::os::raw::c_char;
use std::os::unix::io::AsRawFd;
use std::{
    ffi::{c_void, CStr},
    fs::OpenOptions,
};
use std::{fs::File, os::unix::fs::OpenOptionsExt};
use thiserror::Error;

use super::{
    ffi::{
        is_keyboard_or_mouse, xkb_key_direction, xkb_keycode_t, xkb_keymap_key_repeats, xkb_state,
        xkb_state_get_keymap, xkb_state_key_get_one_sym, xkb_state_key_get_utf8, xkb_state_new,
        xkb_state_unref, xkb_state_update_key, EV_KEY,
    },
    keymap::Keymap,
};

// Read the currently-held keys / lock LEDs straight from an evdev device.
// These expand to `_IOR('E', 0x18/0x19, len)`, i.e. EVIOCGKEY / EVIOCGLED.
nix::ioctl_read_buf!(eviocgkey, b'E', 0x18, u8);
nix::ioctl_read_buf!(eviocgled, b'E', 0x19, u8);

// Momentary modifier keys we seed as "held" at startup, as raw evdev keycodes
// (no xkb offset). The order is significant: each entry maps to one bit of
// `ModifiersState::held`. Left and right are tracked separately so that the
// user's later key-release event (which carries the same code) clears exactly
// the key we pressed, instead of leaving a stand-in modifier stuck down.
const MODIFIER_KEYS: [u32; 8] = [
    29,  // KEY_LEFTCTRL
    97,  // KEY_RIGHTCTRL
    42,  // KEY_LEFTSHIFT
    54,  // KEY_RIGHTSHIFT
    56,  // KEY_LEFTALT
    100, // KEY_RIGHTALT
    125, // KEY_LEFTMETA
    126, // KEY_RIGHTMETA
];

#[derive(Debug, Clone, Copy)]
pub struct ModifiersState {
    // Bitmask over `MODIFIER_KEYS`: bit `i` set means that key is currently held.
    held: u8,
    // Lock states, read from the device LEDs rather than held keys.
    caps_lock: bool,
    num_lock: bool,
}

impl ModifiersState {
    // OR-combine the readings from several devices (parent module folds over these).
    pub(super) fn merge(self, o: Self) -> Self {
        Self {
            held: self.held | o.held,
            caps_lock: self.caps_lock || o.caps_lock,
            num_lock: self.num_lock || o.num_lock,
        }
    }
}

const EVDEV_OFFSET: i32 = 8;
pub const KEY_STATE_RELEASE: i32 = 0;
pub const KEY_STATE_PRESS: i32 = 1;
pub const KEY_STATE_REPEAT: i32 = 2;

#[derive(Debug)]
pub enum RawInputEvent {
    Keyboard(RawKeyboardEvent),
    Mouse(RawMouseEvent),
}

#[derive(Debug)]
pub struct RawKeyboardEvent {
    pub sym: u32,
    pub code: u32,
    pub value: String,
    pub state: i32,
}

#[derive(Debug)]
pub struct RawMouseEvent {
    pub code: u16,
    pub is_down: bool,
}

pub struct Device {
    path: String,
    file: File,
    state: *mut xkb_state,
}

impl Device {
    pub fn from(path: &str, keymap: &Keymap) -> Result<Device> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(O_NONBLOCK | O_CLOEXEC | O_RDONLY)
            .open(path)?;

        if unsafe { is_keyboard_or_mouse(file.as_raw_fd()) == 0 } {
            return Err(DeviceError::InvalidDevice(path.to_string()).into());
        }

        let raw_state = unsafe { xkb_state_new(keymap.get_handle()) };
        // Automatically close the state if the function does not return correctly
        let state = scopeguard::guard(raw_state, |raw_state| unsafe {
            xkb_state_unref(raw_state);
        });

        if raw_state.is_null() {
            return Err(DeviceError::InvalidState(path.to_string()).into());
        }

        Ok(Self {
            path: path.to_string(),
            file,
            // Release the state without freeing it
            state: ScopeGuard::into_inner(state),
        })
    }

    pub fn get_state(&self) -> *mut xkb_state {
        self.state
    }

    pub fn get_raw_fd(&self) -> i32 {
        self.file.as_raw_fd()
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    // Seed the currently-held modifiers at worker startup. evdev only streams
    // future events, so we query the live key/LED state of the already-open fd
    // via ioctl rather than mapping a (focus-stealing, formerly rainbow) window.
    pub fn get_modifiers(&self) -> ModifiersState {
        let fd = self.file.as_raw_fd();
        // Best-effort: non-keyboard devices may reject these ioctls; treat as "nothing held".
        let mut keys = [0u8; 96]; // KEY_CNT / 8 = 768 / 8
        let mut leds = [0u8; 2]; // LED_CNT / 8
        let _ = unsafe { eviocgkey(fd, &mut keys) };
        let _ = unsafe { eviocgled(fd, &mut leds) };

        let is_set = |buf: &[u8], code: usize| buf[code / 8] & (1u8 << (code % 8)) != 0;

        let mut held = 0u8;
        for (i, &code) in MODIFIER_KEYS.iter().enumerate() {
            if is_set(&keys, code as usize) {
                held |= 1u8 << i;
            }
        }

        ModifiersState {
            held,
            caps_lock: is_set(&leds, 1), // LED_CAPSL
            num_lock: is_set(&leds, 0),  // LED_NUML
        }
    }

    pub fn read(&self) -> Result<Vec<RawInputEvent>> {
        let errno_ptr = unsafe { libc::__errno_location() };
        let mut len: ssize_t;
        let mut evs: [input_event; 16] = unsafe { std::mem::zeroed() };
        let mut events = Vec::new();

        loop {
            len = unsafe {
                libc::read(
                    self.file.as_raw_fd(),
                    evs.as_mut_ptr() as *mut c_void,
                    std::mem::size_of_val(&evs),
                )
            };
            if len <= 0 {
                break;
            }

            let nevs: size_t = len as usize / std::mem::size_of::<input_event>();

            #[allow(clippy::needless_range_loop)]
            for i in 0..nevs {
                let event = self.process_event(evs[i].type_, evs[i].code, evs[i].value);
                if let Some(event) = event {
                    events.push(event);
                }
            }
        }

        if len < 0 && unsafe { *errno_ptr } != EWOULDBLOCK {
            if unsafe { *errno_ptr } == ENODEV {
                return Err(DeviceError::FailedReadNoSuchDevice.into());
            }

            return Err(DeviceError::FailedRead(unsafe { *errno_ptr }).into());
        }

        Ok(events)
    }

    fn process_event(&self, _type: u16, code: u16, value: i32) -> Option<RawInputEvent> {
        if _type != EV_KEY {
            return None;
        }

        let is_down = value == KEY_STATE_PRESS;

        // Check if the current event originated from a mouse
        if (0x110..=0x117).contains(&code) {
            // Mouse event
            return Some(RawInputEvent::Mouse(RawMouseEvent { code, is_down }));
        }

        // Keyboard event

        let keycode: xkb_keycode_t = EVDEV_OFFSET as u32 + code as u32;
        let keymap = unsafe { xkb_state_get_keymap(self.get_state()) };

        if value == KEY_STATE_REPEAT && unsafe { xkb_keymap_key_repeats(keymap, keycode) } == 0 {
            return None;
        }

        let sym = unsafe { xkb_state_key_get_one_sym(self.get_state(), keycode) };
        if sym == 0 {
            return None;
        }

        // Extract the utf8 char
        let mut buffer: [c_char; 16] = [0; 16];
        unsafe {
            xkb_state_key_get_utf8(
                self.get_state(),
                keycode,
                buffer.as_mut_ptr(),
                std::mem::size_of_val(&buffer),
            )
        };
        let content_raw = unsafe { CStr::from_ptr(buffer.as_ptr()) };
        let content = content_raw.to_string_lossy().to_string();

        let event = RawKeyboardEvent {
            state: value,
            code: keycode,
            sym,
            value: content,
        };

        if value == KEY_STATE_RELEASE {
            unsafe { xkb_state_update_key(self.get_state(), keycode, xkb_key_direction::UP) };
        } else {
            unsafe { xkb_state_update_key(self.get_state(), keycode, xkb_key_direction::DOWN) };
        }

        Some(RawInputEvent::Keyboard(event))
    }

    pub fn update_key(&mut self, code: u32, pressed: bool) {
        let direction = if pressed {
            super::ffi::xkb_key_direction::DOWN
        } else {
            super::ffi::xkb_key_direction::UP
        };
        unsafe {
            xkb_state_update_key(self.get_state(), code, direction);
        }
    }

    pub fn update_modifier_state(
        &mut self,
        modifiers_state: ModifiersState,
        modifiers_map: &HashMap<String, u32>,
    ) {
        // Press each physically-held modifier by its own keycode (offset into the
        // xkb keymap). Seeding the exact key that is down means the user's later
        // release event clears it, instead of leaving a stand-in modifier stuck.
        for (i, &code) in MODIFIER_KEYS.iter().enumerate() {
            if modifiers_state.held & (1u8 << i) != 0 {
                self.update_key(code + EVDEV_OFFSET as u32, true);
            }
        }

        // Caps/Num lock are lock states (from the LEDs), not held keys: toggle
        // them with a press+release so xkb latches the lock.
        if modifiers_state.num_lock {
            let key = *modifiers_map
                .get("num_lock")
                .expect("unable to find modifiers key in map");
            self.update_key(key, true);
            self.update_key(key, false);
        }
        if modifiers_state.caps_lock {
            let key = *modifiers_map
                .get("caps_lock")
                .expect("unable to find modifiers key in map");
            self.update_key(key, true);
            self.update_key(key, false);
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            xkb_state_unref(self.state);
        }
    }
}

pub fn get_devices(keymap: &Keymap) -> Result<Vec<Device>> {
    let mut keyboards = Vec::new();
    let dirs = std::fs::read_dir("/dev/input/")?;
    for entry in dirs {
        match entry {
            Ok(device) => {
                // Skip non-eventX devices
                if !device.file_name().to_string_lossy().starts_with("event") {
                    continue;
                }

                let path = device.path().to_string_lossy().to_string();
                let keyboard = Device::from(&path, keymap);
                match keyboard {
                    Ok(keyboard) => {
                        keyboards.push(keyboard);
                    }
                    Err(error) => {
                        trace!("error opening keyboard: {}", error);
                    }
                }
            }
            Err(error) => {
                trace!("could not read keyboard device: {}", error);
            }
        }
    }

    if keyboards.is_empty() {
        return Err(DeviceError::NoDevicesFound().into());
    }

    Ok(keyboards)
}

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("could not create xkb state for `{0}`")]
    InvalidState(String),

    #[error("`{0}` is not a valid device")]
    InvalidDevice(String),

    #[error("no devices found")]
    NoDevicesFound(),

    #[error("read operation failed with code: `{0}`")]
    FailedRead(i32),

    #[error("read operation failed: ENODEV No such device")]
    FailedReadNoSuchDevice,
}

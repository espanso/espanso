/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

use std::sync::mpsc::Sender;
use std::os::raw::{c_void, c_char};
use crate::event::*;
use crate::event::KeyModifier::*;
use crate::bridge::linux::*;
use std::process::exit;
use log::{debug, error};
use std::ffi::CStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering::Acquire;
use crate::config::Configs;

#[repr(C)]
pub struct LinuxContext {
    pub send_channel: Sender<Event>,
    is_injecting: Arc<AtomicBool>,
}

impl LinuxContext {
    pub fn new(_: Configs, send_channel: Sender<Event>, is_injecting: Arc<AtomicBool>) -> Box<LinuxContext> {
        // Check if the X11 context is available
        let x11_available = unsafe {
            check_x11()
        };

        if x11_available < 0 {
            error!("Error, can't connect to X11 context");
            std::process::exit(100);
        }

        let context = Box::new(LinuxContext {
            send_channel,
            is_injecting
        });

        unsafe {
            let context_ptr = &*context as *const LinuxContext as *const c_void;

            register_keypress_callback(keypress_callback);

            let res = initialize(context_ptr);
            if res <= 0 {
                error!("Could not initialize linux context, error: {}", res);
                exit(10);
            }
        }

        context
    }
}

impl super::Context for LinuxContext {
    fn eventloop(&self) {
        unsafe {
            eventloop();
        }
    }
}

impl Drop for LinuxContext {
    fn drop(&mut self) {
        unsafe { cleanup(); }
    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut c_void, raw_buffer: *const u8, _len: i32,
                            event_type: i32, key_code: i32) {
    unsafe {
        let _self = _self as *mut LinuxContext;

        // If espanso is currently injecting text, we should avoid processing
        // external events, as it could happen that espanso reinterpret its
        // own input.
        if (*_self).is_injecting.load(Acquire) {
            debug!("Input ignored while espanso is injecting text...");
            return;
        }

        if event_type == 0 {  // Char event
            // Convert the received buffer to a string
            let c_str = CStr::from_ptr(raw_buffer as *const c_char);
            let char_str = c_str.to_str();

            // Send the char through the channel
            match char_str {
                Ok(char_str) => {
                    let event = Event::Key(KeyEvent::Char(char_str.to_owned()));
                    (*_self).send_channel.send(event).unwrap();
                },
                Err(e) => {
                    debug!("Unable to receive char: {}",e);
                },
            }
        }else if event_type == 1 {  // Modifier event

            let modifier: Option<KeyModifier> = match key_code {
                133 => Some(LEFT_META),
                134 => Some(RIGHT_META),
                50 => Some(LEFT_SHIFT),
                62 => Some(RIGHT_SHIFT),
                64 => Some(LEFT_ALT),
                108 => Some(RIGHT_ALT),
                37 => Some(LEFT_CTRL),
                105 => Some(RIGHT_CTRL),
                22 => Some(BACKSPACE),
                66 => Some(CAPS_LOCK),
                _ => None,
            };

            if let Some(modifier) = modifier {
                let event = Event::Key(KeyEvent::Modifier(modifier));
                (*_self).send_channel.send(event).unwrap();
            }else{  // Not one of the default modifiers, send an "other" event
                let event = Event::Key(KeyEvent::Other);
                (*_self).send_channel.send(event).unwrap();
            }
        }else{ // Other type of event
            let event = Event::Key(KeyEvent::Other);
            (*_self).send_channel.send(event).unwrap();
        }
    }
}
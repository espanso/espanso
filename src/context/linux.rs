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
use std::os::raw::c_void;
use crate::event::*;
use crate::event::KeyModifier::*;
use crate::bridge::linux::*;
use std::process::exit;
use log::error;

#[repr(C)]
pub struct LinuxContext {
    pub send_channel: Sender<Event>
}

impl LinuxContext {
    pub fn new(send_channel: Sender<Event>) -> Box<LinuxContext> {
        let context = Box::new(LinuxContext {
            send_channel,
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

extern fn keypress_callback(_self: *mut c_void, raw_buffer: *const u8, len: i32,
                            is_modifier: i32, key_code: i32) {
    unsafe {
        let _self = _self as *mut LinuxContext;

        if is_modifier == 0 {  // Char event
            // Convert the received buffer to a character
            let buffer = std::slice::from_raw_parts(raw_buffer, len as usize);
            let r = String::from_utf8_lossy(buffer).chars().nth(0);

            // Send the char through the channel
            if let Some(c) = r {
                let event = Event::Key(KeyEvent::Char(c));
                (*_self).send_channel.send(event).unwrap();
            }
        }else{  // Modifier event
            let modifier: Option<KeyModifier> = match key_code {
                133 => Some(META),
                50 => Some(SHIFT),
                64 => Some(ALT),
                37 => Some(CTRL),
                22 => Some(BACKSPACE),
                _ => None,
            };

            if let Some(modifier) = modifier {
                let event = Event::Key(KeyEvent::Modifier(modifier));
                (*_self).send_channel.send(event).unwrap();
            }
        }
    }
}
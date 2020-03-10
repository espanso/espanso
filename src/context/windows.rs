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
use crate::bridge::windows::*;
use crate::event::{Event, KeyEvent, KeyModifier, ActionType};
use crate::event::KeyModifier::*;
use std::ffi::c_void;
use std::{fs};
use widestring::{U16CString, U16CStr};
use log::{info, error, debug};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering::Acquire;

const BMP_BINARY : &[u8] = include_bytes!("../res/win/espanso.bmp");
const ICO_BINARY : &[u8] = include_bytes!("../res/win/espanso.ico");

pub struct WindowsContext {
    send_channel: Sender<Event>,
    is_injecting: Arc<AtomicBool>,
}

impl WindowsContext {
    pub fn new(send_channel: Sender<Event>, is_injecting: Arc<AtomicBool>) -> Box<WindowsContext> {
        // Initialize image resources

        let espanso_dir = super::get_data_dir();

        info!("Initializing Espanso resources in {}", espanso_dir.as_path().display());

        let espanso_bmp_image = espanso_dir.join("espansoicon.bmp");
        if espanso_bmp_image.exists() {
            info!("BMP already initialized, skipping.");
        }else {
            fs::write(&espanso_bmp_image, BMP_BINARY)
                .expect("Unable to write windows bmp file");

            info!("Extracted bmp icon to: {}", espanso_bmp_image.to_str().unwrap_or("error"));
        }

        let espanso_ico_image = espanso_dir.join("espanso.ico");
        if espanso_ico_image.exists() {
            info!("ICO already initialized, skipping.");
        }else {
            fs::write(&espanso_ico_image, ICO_BINARY)
                .expect("Unable to write windows ico file");

            info!("Extracted 'ico' icon to: {}", espanso_ico_image.to_str().unwrap_or("error"));
        }

        let bmp_icon = espanso_bmp_image.to_str().unwrap_or_default();
        let ico_icon = espanso_ico_image.to_str().unwrap_or_default();

        let send_channel = send_channel;

        let context = Box::new(WindowsContext{
            send_channel,
            is_injecting,
        });

        unsafe {
            let context_ptr = &*context as *const WindowsContext as *const c_void;

            // Register callbacks
            register_keypress_callback(keypress_callback);
            register_icon_click_callback(icon_click_callback);
            register_context_menu_click_callback(context_menu_click_callback);

            let ico_file_c = U16CString::from_str(ico_icon).unwrap();
            let bmp_file_c = U16CString::from_str(bmp_icon).unwrap();

            // Initialize the windows
            let res = initialize(context_ptr, ico_file_c.as_ptr(), bmp_file_c.as_ptr());
            if res != 1 {
                panic!("Can't initialize Windows context")
            }
        }

        context
    }
}

impl super::Context for WindowsContext {
    fn eventloop(&self) {
        unsafe {
            eventloop();
        }
    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut c_void, raw_buffer: *const u16, len: i32,
                            event_type: i32, key_code: i32, variant: i32, is_key_down: i32) {
    unsafe {
        let _self = _self as *mut WindowsContext;

        // If espanso is currently injecting text, we should avoid processing
        // external events, as it could happen that espanso reinterpret its
        // own input.
        if (*_self).is_injecting.load(Acquire) {
            debug!("Input ignored while espanso is injecting text...");
            return;
        }

        if is_key_down != 0 {  // KEY DOWN EVENT
            if event_type == 0 {  // Char event
                // Convert the received buffer to a string
                let buffer = std::slice::from_raw_parts(raw_buffer, len as usize);
                let c_string = U16CStr::from_slice_with_nul(buffer);

                if let Ok(c_string) = c_string {
                    let string = c_string.to_string();

                    // Send the char through the channel
                    match string {
                        Ok(string) => {
                            let event = Event::Key(KeyEvent::Char(string));
                            (*_self).send_channel.send(event).unwrap();
                        },
                        Err(e) => {
                            error!("Unable to receive char: {}",e);
                        },
                    }
                }else{
                    error!("unable to decode widechar");
                }
            }
        }else{  // KEY UP event
            if event_type == 1 {  // Modifier event
                let modifier: Option<KeyModifier> = match (key_code, variant) {
                    (0x5B, _) => Some(LEFT_META),
                    (0x5C, _) => Some(RIGHT_META),
                    (0x10, 1) => Some(LEFT_SHIFT),
                    (0x10, 2) => Some(RIGHT_SHIFT),
                    (0x12, 1) => Some(LEFT_ALT),
                    (0x12, 2) => Some(RIGHT_ALT),
                    (0x11, 1) => Some(LEFT_CTRL),
                    (0x11, 2) => Some(RIGHT_CTRL),
                    (0x08, _)  => Some(BACKSPACE),
                    _ => None,
                };

                if let Some(modifier) = modifier {
                    let event = Event::Key(KeyEvent::Modifier(modifier));
                    (*_self).send_channel.send(event).unwrap();
                }else{  // Not one of the default modifiers, send an "other" event
                    let event = Event::Key(KeyEvent::Other);
                    (*_self).send_channel.send(event).unwrap();
                }
            }else{
                // Other type of event
                let event = Event::Key(KeyEvent::Other);
                (*_self).send_channel.send(event).unwrap();
            }
        }
    }
}

extern fn icon_click_callback(_self: *mut c_void) {
    unsafe {
        let _self = _self as *mut WindowsContext;

        let event = Event::Action(ActionType::IconClick);
        (*_self).send_channel.send(event).unwrap();
    }
}


extern fn context_menu_click_callback(_self: *mut c_void, id: i32) {
    unsafe {
        let _self = _self as *mut WindowsContext;

        let event = Event::Action(ActionType::from(id));
        (*_self).send_channel.send(event).unwrap();
    }
}
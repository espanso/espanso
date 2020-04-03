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
use crate::bridge::macos::*;
use crate::event::{Event, KeyEvent, KeyModifier, ActionType, SystemEvent};
use crate::event::KeyModifier::*;
use std::ffi::{CString, CStr};
use std::{fs, thread};
use log::{info, error, debug};
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering::Acquire;
use crate::config::Configs;
use std::cell::RefCell;
use crate::system::macos::MacSystemManager;

const STATUS_ICON_BINARY : &[u8] = include_bytes!("../res/mac/icon.png");

pub struct MacContext {
    pub send_channel: Sender<Event>,
    is_injecting: Arc<AtomicBool>,
    secure_input_watcher_enabled: bool,
    secure_input_watcher_interval: i32,
}

impl MacContext {
    pub fn new(config: Configs, send_channel: Sender<Event>, is_injecting: Arc<AtomicBool>) -> Box<MacContext> {
        // Check accessibility
        unsafe {
            let res = prompt_accessibility();

            if res == 0 {
                error!("Accessibility must be enabled to make espanso work on MacOS.");
                error!("Please allow espanso in the Security & Privacy panel, then restart espanso.");
                error!("For more information: https://espanso.org/install/mac/");
                exit(1);
            }
        }

        let context = Box::new(MacContext {
            send_channel,
            is_injecting,
            secure_input_watcher_enabled: config.secure_input_watcher_enabled,
            secure_input_watcher_interval: config.secure_input_watcher_interval,
        });

        // Initialize the status icon path
        let espanso_dir = super::get_data_dir();
        let status_icon_target = espanso_dir.join("icon.png");

        if status_icon_target.exists() {
            info!("Status icon already initialized, skipping.");
        }else {
            fs::write(&status_icon_target, STATUS_ICON_BINARY).unwrap_or_else(|e| {
               error!("Error copying the Status Icon to the espanso data directory: {}", e);
            });
        }

        unsafe {
            let context_ptr = &*context as *const MacContext as *const c_void;

            register_keypress_callback(keypress_callback);
            register_icon_click_callback(icon_click_callback);
            register_context_menu_click_callback(context_menu_click_callback);

            let status_icon_path = CString::new(status_icon_target.to_str().unwrap_or_default()).unwrap_or_default();
            initialize(context_ptr, status_icon_path.as_ptr());
        }

        context
    }

    fn start_secure_input_watcher(&self) {
        let send_channel = self.send_channel.clone();
        let secure_input_watcher_interval = self.secure_input_watcher_interval as u64;

        let secure_input_watcher = thread::Builder::new().name("secure_input_watcher".to_string()).spawn(move || {
            let mut last_secure_input_pid: Option<i64> = None;
            loop {
                let pid = MacSystemManager::get_secure_input_pid();

                if let Some(pid) = pid { // Some application is currently on SecureInput
                    let should_notify = if let Some(old_pid) = last_secure_input_pid { // We already detected a SecureInput app
                        if old_pid != pid { // The old app is different from the current one, we should take action
                            true
                        }else{ // We already notified this application before
                            false
                        }
                    }else{ // First time we see this SecureInput app, we should take action
                        true
                    };

                    if should_notify {
                        let secure_input_app = crate::system::macos::MacSystemManager::get_secure_input_application();

                        if let Some((app_name, path)) = secure_input_app {
                            let event = Event::System(SystemEvent::SecureInputEnabled(app_name, path));
                            send_channel.send(event);
                        }
                    }

                    last_secure_input_pid = Some(pid);
                }else{  // No app is currently keeping SecureInput
                    if let Some(old_pid) = last_secure_input_pid {  // If there was an app with SecureInput, notify that is now free
                        let event = Event::System(SystemEvent::SecureInputDisabled);
                        send_channel.send(event);
                    }

                    last_secure_input_pid = None
                }

                thread::sleep(std::time::Duration::from_millis(secure_input_watcher_interval));
            }
        });
    }
}

impl super::Context for MacContext {
    fn eventloop(&self) {
        // Start the SecureInput watcher thread
        if self.secure_input_watcher_enabled {
            self.start_secure_input_watcher();
        }

        unsafe {
            eventloop();
        }
    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut c_void, raw_buffer: *const u8, len: i32,
                             event_type: i32, key_code: i32) {
    unsafe {
        let _self = _self as *mut MacContext;

        // If espanso is currently injecting text, we should avoid processing
        // external events, as it could happen that espanso reinterpret its
        // own input.
        if (*_self).is_injecting.load(Acquire) {
            debug!("Input ignored while espanso is injecting text...");
            return;
        }

        if event_type == 0 {  // Char event
            // Convert the received buffer to a string
            let c_str = CStr::from_ptr(raw_buffer as (*const c_char));
            let char_str = c_str.to_str();

            // Send the char through the channel
            match char_str {
                Ok(char_str) => {
                    let event = Event::Key(KeyEvent::Char(char_str.to_owned()));
                    (*_self).send_channel.send(event).unwrap();
                },
                Err(e) => {
                    error!("Unable to receive char: {}",e);
                },
            }
        }else if event_type == 1 {  // Modifier event
            let modifier: Option<KeyModifier> = match key_code {
                0x37 => Some(LEFT_META),
                0x36 => Some(RIGHT_META),
                0x38 => Some(LEFT_SHIFT),
                0x3C => Some(RIGHT_SHIFT),
                0x3A => Some(LEFT_ALT),
                0x3D => Some(RIGHT_ALT),
                0x3B => Some(LEFT_CTRL),
                0x3E => Some(RIGHT_CTRL),
                0x33 => Some(BACKSPACE),
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

extern fn icon_click_callback(_self: *mut c_void) {
    unsafe {
        let _self = _self as *mut MacContext;

        let event = Event::Action(ActionType::IconClick);
        (*_self).send_channel.send(event).unwrap();
    }
}

extern fn context_menu_click_callback(_self: *mut c_void, id: i32) {
    unsafe {
        let _self = _self as *mut MacContext;

        let event = Event::Action(ActionType::from(id));
        (*_self).send_channel.send(event).unwrap();
    }
}

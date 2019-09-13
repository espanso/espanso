use std::sync::mpsc::Sender;
use std::os::raw::c_void;
use crate::bridge::macos::*;
use crate::event::{Event, KeyEvent, KeyModifier, ActionEvent, ActionType};
use crate::event::KeyModifier::*;
use std::fs::create_dir_all;
use std::ffi::CString;
use std::fs;
use log::{info};

const STATUS_ICON_BINARY : &'static [u8] = include_bytes!("../res/mac/icon.png");

pub struct MacContext {
    pub send_channel: Sender<Event>
}

impl MacContext {
    pub fn new(send_channel: Sender<Event>) -> Box<MacContext> {
        let context = Box::new(MacContext {
           send_channel
        });

        // Initialize the status icon path
        let data_dir = dirs::data_dir().expect("Can't obtain data_dir(), terminating.");
        let espanso_dir = data_dir.join("espanso");
        let res = create_dir_all(&espanso_dir);
        let status_icon_target = espanso_dir.join("icon.png");

        if status_icon_target.exists() {
            info!("Status icon already initialized, skipping.");
        }else {
            fs::write(&status_icon_target, STATUS_ICON_BINARY);
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
}

impl super::Context for MacContext {
    fn eventloop(&self) {
        unsafe {
            eventloop();
        }
    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut c_void, raw_buffer: *const u8, len: i32,
                             is_modifier: i32, key_code: i32) {
    unsafe {
        let _self = _self as *mut MacContext;

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
                0x37 => Some(META),
                0x38 => Some(SHIFT),
                0x3A => Some(ALT),
                0x3B => Some(CTRL),
                0x33 => Some(BACKSPACE),
                _ => None,
            };

            if let Some(modifier) = modifier {
                let event = Event::Key(KeyEvent::Modifier(modifier));
                (*_self).send_channel.send(event).unwrap();
            }
        }
    }
}

extern fn icon_click_callback(_self: *mut c_void) {
    unsafe {
        let _self = _self as *mut MacContext;

        let event = Event::Action(ActionEvent::IconClick);
        (*_self).send_channel.send(event).unwrap();
    }
}

extern fn context_menu_click_callback(_self: *mut c_void, id: i32) {
    unsafe {
        let _self = _self as *mut MacContext;

        let event = Event::Action(ActionEvent::ContextMenuClick(ActionType::from(id)));
        (*_self).send_channel.send(event).unwrap();
    }
}
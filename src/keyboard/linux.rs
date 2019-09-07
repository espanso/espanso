use std::{thread};
use std::sync::mpsc;
use std::os::raw::{c_char, c_void};
use std::ffi::CString;
use crate::keyboard::{KeyEvent, KeyModifier};
use crate::keyboard::KeyModifier::*;

use crate::bridge::linux::*;

#[repr(C)]
pub struct LinuxKeyboardInterceptor {
    pub sender: mpsc::Sender<KeyEvent>
}

impl super::KeyboardInterceptor for LinuxKeyboardInterceptor {
    fn initialize(&self) {
        unsafe {
            let self_ptr = self as *const LinuxKeyboardInterceptor as *const c_void;
            register_keypress_callback( self_ptr,keypress_callback);
            initialize();  // TODO: check initialization return codes
        }
    }

    fn start(&self) {
        unsafe {
            eventloop();
        }
    }
}

impl Drop for LinuxKeyboardInterceptor {
    fn drop(&mut self) {
        unsafe { cleanup(); }
    }
}

pub struct LinuxKeyboardSender {
}

impl super::KeyboardSender for LinuxKeyboardSender {
    fn send_string(&self, s: &str) {
        let res = CString::new(s);
        match res {
            Ok(cstr) => unsafe { send_string(cstr.as_ptr()); }
            Err(e) => panic!(e.to_string())
        }
    }

    fn send_enter(&self) {
        // On linux this is not needed, so NOOP
    }

    fn trigger_paste(&self) {
        unsafe { trigger_paste(); }

        // TODO: detect when in terminal and use trigger_terminal_paste() instead
    }

    fn delete_string(&self, count: i32) {
        unsafe {delete_string(count)}
    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut c_void, raw_buffer: *const u8, len: i32,
                            is_modifier: i32, key_code: i32) {
    unsafe {
        let _self = _self as *mut LinuxKeyboardInterceptor;

        if is_modifier == 0 {  // Char event
            // Convert the received buffer to a character
            let buffer = std::slice::from_raw_parts(raw_buffer, len as usize);
            let r = String::from_utf8_lossy(buffer).chars().nth(0);

            // Send the char through the channel
            if let Some(c) = r {
                (*_self).sender.send(KeyEvent::Char(c)).unwrap();
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
                (*_self).sender.send(KeyEvent::Modifier(modifier)).unwrap();
            }
        }
    }
}
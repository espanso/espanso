use std::sync::mpsc;
use widestring::{U16CString};
use crate::keyboard::{KeyEvent, KeyModifier};
use crate::keyboard::KeyModifier::*;

#[repr(C)]
pub struct WindowsKeyboardInterceptor {
    pub sender: mpsc::Sender<KeyEvent>
}

impl super::KeyboardInterceptor for WindowsKeyboardInterceptor {
    fn initialize(&self) {
        unsafe {
            register_keypress_callback(self,keypress_callback);
            initialize_window();
        }
    }

    fn start(&self) {
        unsafe {
            eventloop();
        }
    }
}

pub struct WindowsKeyboardSender {
}

impl super::KeyboardSender for WindowsKeyboardSender {
    fn send_string(&self, s: &str) {
        let res = U16CString::from_str(s);
        match res {
            Ok(s) => {
                unsafe {
                    send_string(s.as_ptr());
                }
            }
            Err(e) => println!("Error while sending string: {}", e.to_string())
        }

    }

    fn delete_string(&self, count: i32) {
        unsafe {
            delete_string(count)
        }
    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut WindowsKeyboardInterceptor, raw_buffer: *const i32, len: i32,
                            is_modifier: i32, key_code: i32) {
    unsafe {
        if is_modifier == 0 {  // Char event
            // Convert the received buffer to a character
            let buffer = std::slice::from_raw_parts(raw_buffer, len as usize);
            let r = std::char::from_u32(buffer[0] as u32);

            // Send the char through the channel
            if let Some(c) = r {
                (*_self).sender.send(KeyEvent::Char(c)).unwrap();
            }
        }else{  // Modifier event
            let modifier: Option<KeyModifier> = match key_code {
                0x5B | 0x5C => Some(META),
                0x10 => Some(SHIFT),
                0x12 => Some(ALT),
                0x11 => Some(CTRL),
                0x08  => Some(BACKSPACE),
                _ => None,
            };

            if let Some(modifier) = modifier {
                (*_self).sender.send(KeyEvent::Modifier(modifier)).unwrap();
            }
        }
    }
}

#[allow(improper_ctypes)]
#[link(name="winbridge", kind="static")]
extern {
    fn register_keypress_callback(s: *const WindowsKeyboardInterceptor,
                                  cb: extern fn(_self: *mut WindowsKeyboardInterceptor, *const i32,
                                                i32, i32, i32));
    fn initialize_window();
    fn eventloop();
    fn send_string(string: *const u16);
    fn delete_string(count: i32);
}
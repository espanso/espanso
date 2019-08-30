use std::thread;
use std::sync::mpsc;
use widestring::{WideString, WideStr};

#[repr(C)]
pub struct WindowsKeyboardInterceptor {
    pub sender: mpsc::Sender<char>
}

impl super::KeyboardInterceptor for WindowsKeyboardInterceptor {
    fn initialize(&self) {
        unsafe {
            register_keypress_callback(self,keypress_callback);
        }
    }

    fn start(&self) {
        thread::spawn(|| {
            unsafe {
                initialize_window();
                eventloop();
            }
        });
    }
}

pub struct WindowsKeyboardSender {
}

impl super::KeyboardSender for WindowsKeyboardSender {
    fn send_string(&self, s: &str) {
        let s = WideString::from(s.to_owned());
        unsafe {
            send_string(s.as_ptr());
        }
    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut WindowsKeyboardInterceptor, raw_buffer: *const i32, len: i32) {
    unsafe {
        // Convert the received buffer to a character
        let buffer = std::slice::from_raw_parts(raw_buffer, len as usize);
        let r = std::char::from_u32(buffer[0] as u32).unwrap();

        // Send the char through the channel
        (*_self).sender.send(r).unwrap();
    }
}

#[link(name="winbridge", kind="static")]
extern {
    fn register_keypress_callback(s: *const WindowsKeyboardInterceptor, cb: extern fn(_self: *mut WindowsKeyboardInterceptor, *const i32, i32));
    fn initialize_window();
    fn eventloop();
    fn send_string(string: *const u16);
}
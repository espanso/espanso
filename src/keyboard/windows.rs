use std::sync::mpsc;
use std::os::raw::{c_void};
use widestring::{U16CString};
use crate::bridge::windows::*;

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

    fn send_enter(&self) {
        unsafe {
            // Send the VK_RETURN key press
            send_vkey(0x0D);
        }
    }

    fn trigger_paste(&self) {
        unimplemented!()
    }

    fn delete_string(&self, count: i32) {
        unsafe {
            delete_string(count)
        }
    }
}
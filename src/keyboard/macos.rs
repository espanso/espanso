use std::ffi::CString;
use crate::bridge::macos::*;

pub struct MacKeyboardManager {
}

impl super::KeyboardManager for MacKeyboardManager {
    fn send_string(&self, s: &str) {
        let res = CString::new(s);
        match res {
            Ok(cstr) => unsafe { send_string(cstr.as_ptr()); }
            Err(e) => panic!(e.to_string())
        }
    }

    fn send_enter(&self) {
        unsafe {
            // Send the kVK_Return key press
            send_vkey(0x24);
        }
    }

    fn trigger_paste(&self) {
        unsafe {
            trigger_paste();
        }
    }

    fn delete_string(&self, count: i32) {
        unsafe {delete_string(count)}
    }
}
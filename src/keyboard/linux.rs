use std::sync::mpsc;
use std::os::raw::{c_void};
use std::ffi::CString;
use crate::bridge::linux::*;

pub struct LinuxKeyboardManager {
}

impl super::KeyboardManager for LinuxKeyboardManager {
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
        unsafe {
            let is_terminal = is_current_window_terminal();

            // Terminals use a different keyboard combination to paste from clipboard,
            // so we need to check the correct situation.
            if is_terminal == 0 {
                trigger_paste();
            }else{
                trigger_terminal_paste();
            }
        }
    }

    fn delete_string(&self, count: i32) {
        unsafe {delete_string(count)}
    }
}
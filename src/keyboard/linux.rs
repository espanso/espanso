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
        unsafe { trigger_paste(); }

        // TODO: detect when in terminal and use trigger_terminal_paste() instead
    }

    fn delete_string(&self, count: i32) {
        unsafe {delete_string(count)}
    }
}
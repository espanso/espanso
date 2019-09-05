use std::sync::mpsc;
use std::os::raw::c_char;
use std::ffi::CString;

#[repr(C)]
pub struct MacKeyboardInterceptor {
    pub sender: mpsc::Sender<char>
}

impl super::KeyboardInterceptor for MacKeyboardInterceptor {
    fn initialize(&self) {
        unsafe { initialize(); }  // TODO: check initialization return codes
    }

    fn start(&self) {
        unsafe { eventloop(); }
    }
}

pub struct MacKeyboardSender {
}

impl super::KeyboardSender for MacKeyboardSender {
    fn send_string(&self, s: &str) {

    }

    fn delete_string(&self, count: i32) {

    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut MacKeyboardInterceptor, raw_buffer: *const u8, len: i32) {
    unsafe {

    }
}

#[allow(improper_ctypes)]
#[link(name="macbridge", kind="static")]
extern {
    fn initialize();
    fn eventloop();
}
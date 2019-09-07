use std::os::raw::{c_void, c_char};

#[allow(improper_ctypes)]
#[link(name="linuxbridge", kind="static")]
extern {
    // System
    pub fn get_active_window_name(buffer: *mut c_char, size: i32) -> i32;
    pub fn get_active_window_class(buffer: *mut c_char, size: i32) -> i32;

    // Keyboard
    pub fn register_keypress_callback(s: *const c_void,
                                  cb: extern fn(_self: *mut c_void, *const u8,
                                                i32, i32, i32));
    pub fn initialize();
    pub fn eventloop();
    pub fn cleanup();
    pub fn send_string(string: *const c_char);
    pub fn delete_string(count: i32);
    pub fn trigger_paste();
    pub fn trigger_terminal_paste();
}
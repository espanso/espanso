use std::os::raw::{c_void, c_char};

#[allow(improper_ctypes)]
#[link(name="linuxbridge", kind="static")]
extern {
    pub fn initialize(s: *const c_void) -> i32;
    pub fn eventloop();
    pub fn cleanup();

    // System
    pub fn get_active_window_name(buffer: *mut c_char, size: i32) -> i32;
    pub fn get_active_window_class(buffer: *mut c_char, size: i32) -> i32;
    pub fn get_active_window_executable(buffer: *mut c_char, size: i32) -> i32;
    pub fn is_current_window_terminal() -> i32;

    // Keyboard
    pub fn register_keypress_callback(cb: extern fn(_self: *mut c_void, *const u8,
                                                i32, i32, i32));

    pub fn send_string(string: *const c_char);
    pub fn delete_string(count: i32);
    pub fn trigger_paste();
    pub fn trigger_terminal_paste();
}
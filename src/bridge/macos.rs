use std::os::raw::{c_void, c_char};

#[allow(improper_ctypes)]
#[link(name="macbridge", kind="static")]
extern {
    pub fn initialize(s: *const c_void);
    pub fn eventloop();

    // System
    pub fn get_active_app_bundle(buffer: *mut c_char, size: i32) -> i32;
    pub fn get_active_app_identifier(buffer: *mut c_char, size: i32) -> i32;

    // Clipboard
    pub fn get_clipboard(buffer: *mut c_char, size: i32) -> i32;
    pub fn set_clipboard(text: *const c_char) -> i32;

    // Keyboard
    pub fn register_keypress_callback(cb: extern fn(_self: *mut c_void, *const u8,
                                                i32, i32, i32));

    pub fn send_string(string: *const c_char);
    pub fn send_vkey(vk: i32);
    pub fn delete_string(count: i32);
    pub fn trigger_paste();
}
use std::os::raw::{c_void};

#[allow(improper_ctypes)]
#[link(name="winbridge", kind="static")]
extern {
    // SYSTEM
    pub fn get_active_window_name(buffer: *mut u16, size: i32) -> i32;
    pub fn get_active_window_executable(buffer: *mut u16, size: i32) -> i32;

    // UI
    pub fn show_notification(message: *const u16, icon_path: *const u16) -> i32;

    // KEYBOARD
    pub fn register_keypress_callback(s: *const c_void,
                                  cb: extern fn(_self: *mut c_void, *const i32,
                                                i32, i32, i32));
    pub fn initialize_window();
    pub fn eventloop();
    pub fn send_string(string: *const u16);
    pub fn send_vkey(vk: i32);
    pub fn delete_string(count: i32);
}
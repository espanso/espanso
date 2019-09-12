use std::os::raw::{c_void};

#[repr(C)]
pub struct WindowsMenuItem {
    pub item_id: i32,
    pub item_type: i32,
    pub item_name: [u16; 100],
}

#[allow(improper_ctypes)]
#[link(name="winbridge", kind="static")]
extern {
    pub fn initialize(s: *const c_void, ico_path: *const u16, bmp_path: *const u16) -> i32;

    // SYSTEM
    pub fn get_active_window_name(buffer: *mut u16, size: i32) -> i32;
    pub fn get_active_window_executable(buffer: *mut u16, size: i32) -> i32;

    // UI
    pub fn show_notification(message: *const u16) -> i32;
    pub fn close_notification();
    pub fn show_context_menu(items: *const WindowsMenuItem, count: i32) -> i32;
    pub fn register_icon_click_callback(cb: extern fn(_self: *mut c_void));
    pub fn register_context_menu_click_callback(cb: extern fn(_self: *mut c_void, id: i32));

    // KEYBOARD
    pub fn register_keypress_callback(cb: extern fn(_self: *mut c_void, *const i32,
                                                i32, i32, i32));

    pub fn eventloop();
    pub fn send_string(string: *const u16);
    pub fn send_vkey(vk: i32);
    pub fn delete_string(count: i32);
}
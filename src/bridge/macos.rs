use std::os::raw::{c_void, c_char};

#[allow(improper_ctypes)]
#[link(name="macbridge", kind="static")]
extern {
    pub fn register_keypress_callback(s: *const c_void,
                                  cb: extern fn(_self: *mut c_void, *const u8,
                                                i32, i32, i32));
    pub fn initialize();
    pub fn eventloop();
    pub fn send_string(string: *const c_char);
    pub fn send_vkey(vk: i32);
    pub fn delete_string(count: i32);
}
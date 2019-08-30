use std::thread;
use std::sync::mpsc;

#[repr(C)]
pub struct WindowsKeyboardBackend {
    pub sender: mpsc::Sender<char>
}

impl super::KeyboardBackend for WindowsKeyboardBackend {
    fn initialize(&self) {
        unsafe {
            register_keypress_callback(self,keypress_callback);
        }
    }

    fn start(&self) {
        thread::spawn(|| {
            unsafe {
                initialize_window();
                eventloop();
            }
        });
    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut WindowsKeyboardBackend, raw_buffer: *const i32, len: i32) {
    unsafe {
        // Convert the received buffer to a character
        let buffer = std::slice::from_raw_parts(raw_buffer, len as usize);
        let r = std::char::from_u32(buffer[0] as u32).unwrap();

        // Send the char through the channel
        (*_self).sender.send(r).unwrap();
    }
}

#[link(name="winbridge", kind="static")]
extern {
    fn register_keypress_callback(s: *const WindowsKeyboardBackend, cb: extern fn(_self: *mut WindowsKeyboardBackend, *const i32, i32));
    fn initialize_window();
    fn eventloop();
}
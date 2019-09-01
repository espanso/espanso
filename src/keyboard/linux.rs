use std::thread;
use std::sync::mpsc;

#[repr(C)]
pub struct LinuxKeyboardInterceptor {
    pub sender: mpsc::Sender<char>
}

impl super::KeyboardInterceptor for LinuxKeyboardInterceptor {
    fn initialize(&self) {
        unsafe {
            register_keypress_callback(self,keypress_callback);
        }
    }

    fn start(&self) {
        thread::spawn(|| {
            unsafe {
                initialize();
                eventloop();
            }
        });
    }
}

pub struct LinuxKeyboardSender {
}

impl super::KeyboardSender for LinuxKeyboardSender {
    fn send_string(&self, s: &str) {
        println!("{}", s);

    }

    fn delete_string(&self, count: i32) {

    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut LinuxKeyboardInterceptor, raw_buffer: *const u8, len: i32) {
    unsafe {
        // Convert the received buffer to a character
        let buffer = std::slice::from_raw_parts(raw_buffer, len as usize);
        let r = String::from_utf8_lossy(buffer).chars().nth(0);

        // Send the char through the channel
        if let Some(c) = r {
            //println!("'{}'",c);
            (*_self).sender.send(c).unwrap();
        }
    }
}

#[allow(improper_ctypes)]
#[link(name="linuxbridge", kind="static")]
extern {
    fn register_keypress_callback(s: *const LinuxKeyboardInterceptor, cb: extern fn(_self: *mut LinuxKeyboardInterceptor, *const u8, i32));
    fn initialize();
    fn eventloop();
    fn cleanup();
}
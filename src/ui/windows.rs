use std::process::Command;
use crate::bridge::windows::{show_notification, close_notification, WindowsMenuItem};
use widestring::U16CString;
use std::{fs, thread, time};
use log::{info, debug};
use std::sync::Mutex;
use std::sync::Arc;
use std::fs::create_dir_all;
use std::os::raw::c_void;

const BMP_BINARY : &'static [u8] = include_bytes!("../res/win/espanso.bmp");
const ICO_BINARY : &'static [u8] = include_bytes!("../res/win/espanso.ico");

pub struct WindowsUIManager {
    id: Arc<Mutex<i32>>
}

impl super::UIManager for WindowsUIManager {
    fn notify(&self, message: &str) {
        let current_id: i32 = {
            let mut id = self.id.lock().unwrap();
            *id += 1;
            *id
        };

        // Setup a timeout to close the notification
        let id = Arc::clone(&self.id);
        thread::spawn(move || {
            for i in 1..10 {
                let duration = time::Duration::from_millis(200);
                thread::sleep(duration);

                let new_id = id.lock().unwrap();
                if *new_id != current_id {
                    debug!("Cancelling notification close event with id {}", current_id);
                    return;
                }
            }

            unsafe {
                close_notification();
            }
        });

        // Create and show a window notification
        unsafe {
            let message = U16CString::from_str(message).unwrap();
            show_notification(message.as_ptr());
        }

    }
}

impl WindowsUIManager {
    pub fn new() -> WindowsUIManager {
        let id = Arc::new(Mutex::new(0));

        let manager = WindowsUIManager {
            id
        };

        manager
    }
}

// NATIVE

extern fn menu_item_callback(_self: *mut c_void, items: *mut WindowsMenuItem, count: *mut i32) {
    unsafe {
        let _self = _self as *mut WindowsUIManager;

        let str = U16CString::from_str("Test").unwrap_or_default();
        let mut str_buff : [u16; 100] = [0; 100];
        std::ptr::copy(str.as_ptr(), str_buff.as_mut_ptr(), str.len());
        let item = WindowsMenuItem {
            item_id: 1,
            item_type: 1,
            item_name: str_buff,
        };

        let items = unsafe { std::slice::from_raw_parts_mut(items, 100) };

        items[0] = item;
        *count = 1;
    }
}
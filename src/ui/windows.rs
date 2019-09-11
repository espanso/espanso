use std::process::Command;
use crate::bridge::windows::{show_notification, close_notification, initialize_ui, WindowsMenuItem, register_menu_item_callback};
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
        let data_dir = dirs::data_dir().expect("Can't obtain data_dir(), terminating.");

        let espanso_dir = data_dir.join("espanso");

        let res = create_dir_all(&espanso_dir);

        info!("Initializing Espanso resources in {}", espanso_dir.as_path().display());

        let espanso_bmp_image = espanso_dir.join("espansoicon.bmp");
        if espanso_bmp_image.exists() {
            info!("BMP already initialized, skipping.");
        }else {
            fs::write(&espanso_bmp_image, BMP_BINARY)
                .expect("Unable to write windows bmp file");

            info!("Extracted bmp icon to: {}", espanso_bmp_image.to_str().unwrap_or("error"));
        }

        let espanso_ico_image = espanso_dir.join("espanso.ico");
        if espanso_ico_image.exists() {
            info!("ICO already initialized, skipping.");
        }else {
            fs::write(&espanso_ico_image, ICO_BINARY)
                .expect("Unable to write windows ico file");

            info!("Extracted 'ico' icon to: {}", espanso_ico_image.to_str().unwrap_or("error"));
        }

        let bmp_icon = espanso_bmp_image.to_str().unwrap_or_default();
        let ico_icon = espanso_ico_image.to_str().unwrap_or_default();

        let id = Arc::new(Mutex::new(0));
        let ico_file_c = U16CString::from_str(ico_icon).unwrap();
        let bmp_file_c = U16CString::from_str(bmp_icon).unwrap();

        let manager = WindowsUIManager {
            id
        };

        // Setup the menu item callback
        unsafe {
            let self_ptr = &manager as *const WindowsUIManager as *const c_void;
            register_menu_item_callback(self_ptr, menu_item_callback);
        }

        thread::spawn(move || {
            unsafe {
                initialize_ui(ico_file_c.as_ptr(), bmp_file_c.as_ptr());
            }
        });

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
use std::process::Command;
use crate::bridge::windows::{show_notification, close_notification, initialize_ui};
use widestring::U16CString;
use std::{fs, thread, time};
use log::{info, debug};
use std::sync::Mutex;
use std::sync::Arc;

const ICON_BINARY : &'static [u8] = include_bytes!("../res/win/espanso.bmp");

pub struct WindowsUIManager {
    icon_file: String,
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
        let res = dirs::cache_dir();
        let mut icon_file:String = "".to_owned();
        if let Some(cache_dir) = res {
            let espanso_icon_file = cache_dir.join("espansoicon.bmp");

            fs::write(&espanso_icon_file, ICON_BINARY)
                .expect("Unable to write windows icon file");

            icon_file = espanso_icon_file.to_str().unwrap_or(&"".to_owned()).to_owned();
        }

        info!("Extracted cached icon to: {}", icon_file);

        let id = Arc::new(Mutex::new(0));
        let icon_file_c = U16CString::from_str(&icon_file).unwrap();

        thread::spawn(move || {
            unsafe {
                initialize_ui(icon_file_c.as_ptr());
            }
        });

        WindowsUIManager {
            icon_file,
            id
        }
    }
}
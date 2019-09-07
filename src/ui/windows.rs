use std::process::Command;
use crate::bridge::windows::show_notification;
use widestring::U16CString;
use std::fs;
use log::{info};

const ICON_BINARY : &'static [u8] = include_bytes!("../res/win/espanso.bmp");

pub struct WindowsUIManager {
    icon_file: String,
}

impl super::UIManager for WindowsUIManager {
    fn notify(&self, message: &str) {
        unsafe {
            let message = U16CString::from_str(message).unwrap();
            let icon_file = U16CString::from_str(&self.icon_file).unwrap();
            let res = show_notification(message.as_ptr(), icon_file.as_ptr());
            info!("{}", res);
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

        WindowsUIManager {
            icon_file
        }
    }
}
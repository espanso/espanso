use std::process::Command;
use super::MenuItem;
use log::error;

pub struct LinuxUIManager {}

impl super::UIManager for LinuxUIManager {
    fn notify(&self, message: &str) {
        let res = Command::new("notify-send")
                        .args(&["-t", "2000", "espanso", message])
                        .output();

        if let Err(e) = res {
            error!("Could not send a notification, error: {}", e);
        }
    }

    fn show_menu(&self, _menu: Vec<MenuItem>) {
        // Not implemented on linux
    }
}

impl LinuxUIManager {
    pub fn new() -> LinuxUIManager {
        LinuxUIManager{}
    }
}
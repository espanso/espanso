use std::process::Command;
use super::MenuItem;

pub struct LinuxUIManager {}

impl super::UIManager for LinuxUIManager {
    fn notify(&self, message: &str) {
        let res = Command::new("notify-send")
                        .args(&["-t", "2000", "espanso", message])
                        .output();

        if let Err(_) = res {
            // TODO: print error log
        }
    }

    fn show_menu(&self, menu: Vec<MenuItem>) {
        unimplemented!()
    }
}

impl LinuxUIManager {
    pub fn new() -> LinuxUIManager {
        // TODO: check if notify-send is present and log an error otherwise.

        LinuxUIManager{}
    }
}
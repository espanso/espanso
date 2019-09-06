use std::process::Command;

pub struct LinuxUIManager {

}

impl super::UIManager for LinuxUIManager {
    fn initialize(&self) {
        // TODO: check if notify-send is present and log an error otherwise.
    }

    fn notify(&self, message: &str) {
        let res = Command::new("notify-send")
                        .args(&["-t", "2000", "espanso", message])
                        .output();

        if let Err(_) = res {
            // TODO: print error log
        }
    }
}

impl LinuxUIManager {

}
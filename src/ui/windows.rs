use std::process::Command;

pub struct WindowsUIManager {

}

impl super::UIManager for WindowsUIManager {
    fn initialize(&self) {
        // TODO: check if notify-send is present and log an error otherwise.
    }

    fn notify(&self, message: &str) {

    }
}

impl WindowsUIManager {

}
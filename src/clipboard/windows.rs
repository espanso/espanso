use std::process::{Command, Stdio};
use std::io::{Write};

pub struct WindowsClipboardManager {

}

impl WindowsClipboardManager {
    pub fn new() -> WindowsClipboardManager {
        WindowsClipboardManager{}
    }
}

impl super::ClipboardManager for WindowsClipboardManager {
    fn get_clipboard(&self) -> Option<String>  {
        unimplemented!();
    }

    fn set_clipboard(&self, payload: &str) {
        unimplemented!();
    }
}
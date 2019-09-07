use std::process::{Command, Stdio};
use std::io::{Write};

pub struct WindowsClipboardManager {

}

impl super::ClipboardManager for WindowsClipboardManager {
    fn initialize(&self) {
        // TODO: check if xclip is present and log an error otherwise.
    }

    fn get_clipboard(&self) -> Option<String>  {
        unimplemented!();
    }

    fn set_clipboard(&self, payload: &str) {
        unimplemented!();
    }
}

impl WindowsClipboardManager {

}
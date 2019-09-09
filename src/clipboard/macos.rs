pub struct MacClipboardManager {

}

impl super::ClipboardManager for MacClipboardManager {
    fn get_clipboard(&self) -> Option<String>  {
        unimplemented!();
    }

    fn set_clipboard(&self, payload: &str) {
        unimplemented!();
    }
}

impl MacClipboardManager {
    pub fn new() -> MacClipboardManager {
        MacClipboardManager{}
    }
}
use std::os::raw::c_char;
use crate::bridge::macos::{get_clipboard, set_clipboard};
use std::ffi::{CStr, CString};

pub struct MacClipboardManager {

}

impl super::ClipboardManager for MacClipboardManager {
    fn get_clipboard(&self) -> Option<String>  {
        unsafe {
            let mut buffer : [c_char; 2000] = [0; 2000];
            let res = get_clipboard(buffer.as_mut_ptr(), buffer.len() as i32);

            if res > 0 {
                let c_string = CStr::from_ptr(buffer.as_ptr());

                let string = c_string.to_str();
                if let Ok(string) = string {
                    return Some((*string).to_owned());
                }
            }
        }

        None
    }

    fn set_clipboard(&self, payload: &str) {
        let res = CString::new(payload);
        if let Ok(cstr) = res {
            unsafe {
                set_clipboard(cstr.as_ptr());
            }
        }
    }
}

impl MacClipboardManager {
    pub fn new() -> MacClipboardManager {
        MacClipboardManager{}
    }
}
use std::os::raw::c_char;

use crate::bridge::linux::{get_active_window_name, get_active_window_class};
use std::ffi::CStr;

pub struct LinuxSystemManager {

}

impl super::SystemManager for LinuxSystemManager {
    fn get_current_window_title(&self) -> Option<String> {
        unsafe {
            let mut buffer : [c_char; 100] = [0; 100];
            let res = get_active_window_name(buffer.as_mut_ptr(), buffer.len() as i32);

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

    fn get_current_window_class(&self) -> Option<String> {
        unsafe {
            let mut buffer : [c_char; 100] = [0; 100];
            let res = get_active_window_class(buffer.as_mut_ptr(), buffer.len() as i32);

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

    fn get_current_window_executable(&self) -> Option<String> {
        unimplemented!()
    }
}

unsafe impl Send for LinuxSystemManager {}

impl LinuxSystemManager {

}
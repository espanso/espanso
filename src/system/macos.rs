use std::os::raw::c_char;

use std::ffi::CStr;

pub struct MacSystemManager {

}

impl super::SystemManager for MacSystemManager {
    fn get_current_window_title(&self) -> Option<String> {
        unimplemented!()
    }

    fn get_current_window_class(&self) -> Option<String> {
        unimplemented!();
    }

    fn get_current_window_executable(&self) -> Option<String> {
        unimplemented!()
    }
}

impl MacSystemManager {
    pub fn new() -> MacSystemManager {
        MacSystemManager{

        }
    }
}
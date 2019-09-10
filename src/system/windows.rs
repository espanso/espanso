use widestring::U16CString;
use crate::bridge::windows::*;

pub struct WindowsSystemManager {

}

impl WindowsSystemManager {
    pub fn new() -> WindowsSystemManager {
        WindowsSystemManager{}
    }
}

impl super::SystemManager for WindowsSystemManager {
    fn get_current_window_title(&self) -> Option<String> {
        unsafe {
            let mut buffer : [u16; 100] = [0; 100];
            let res = get_active_window_name(buffer.as_mut_ptr(), buffer.len() as i32);

            if res > 0 {
                let c_string = U16CString::from_ptr_str(buffer.as_ptr());

                let string = c_string.to_string_lossy();
                return Some((*string).to_owned());
            }
        }

        None
    }

    fn get_current_window_class(&self) -> Option<String> {
        self.get_current_window_executable()
    }

    fn get_current_window_executable(&self) -> Option<String> {
        unsafe {
            let mut buffer : [u16; 250] = [0; 250];
            let res = get_active_window_executable(buffer.as_mut_ptr(), buffer.len() as i32);

            if res > 0 {
                let c_string = U16CString::from_ptr_str(buffer.as_ptr());

                let string = c_string.to_string_lossy();
                return Some((*string).to_owned());
            }
        }

        None
    }
}
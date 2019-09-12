use std::sync::mpsc::Sender;
use crate::bridge::windows::*;
use crate::event::{Event, KeyEvent, KeyModifier};
use crate::event::KeyModifier::*;
use std::ffi::c_void;
use std::fs::create_dir_all;
use std::{fs, thread, time};
use std::sync::{Arc, Mutex};
use widestring::U16CString;
use log::{info, debug, error};

const BMP_BINARY : &'static [u8] = include_bytes!("../res/win/espanso.bmp");
const ICO_BINARY : &'static [u8] = include_bytes!("../res/win/espanso.ico");

pub struct WindowsContext {
    send_channel: Sender<Event>,
    id: Arc<Mutex<i32>>
}

impl WindowsContext {
    pub fn new(send_channel: Sender<Event>) -> Box<WindowsContext> {
        // Initialize image resources

        let data_dir = dirs::data_dir().expect("Can't obtain data_dir(), terminating.");
        let espanso_dir = data_dir.join("espanso");
        let res = create_dir_all(&espanso_dir);

        info!("Initializing Espanso resources in {}", espanso_dir.as_path().display());

        let espanso_bmp_image = espanso_dir.join("espansoicon.bmp");
        if espanso_bmp_image.exists() {
            info!("BMP already initialized, skipping.");
        }else {
            fs::write(&espanso_bmp_image, BMP_BINARY)
                .expect("Unable to write windows bmp file");

            info!("Extracted bmp icon to: {}", espanso_bmp_image.to_str().unwrap_or("error"));
        }

        let espanso_ico_image = espanso_dir.join("espanso.ico");
        if espanso_ico_image.exists() {
            info!("ICO already initialized, skipping.");
        }else {
            fs::write(&espanso_ico_image, ICO_BINARY)
                .expect("Unable to write windows ico file");

            info!("Extracted 'ico' icon to: {}", espanso_ico_image.to_str().unwrap_or("error"));
        }

        let bmp_icon = espanso_bmp_image.to_str().unwrap_or_default();
        let ico_icon = espanso_ico_image.to_str().unwrap_or_default();

        let id = Arc::new(Mutex::new(0));
        let send_channel = send_channel;

        let manager = Box::new(WindowsContext{
            send_channel,
            id
        });

        unsafe {
            let manager_ptr = &*manager as *const WindowsContext as *const c_void;

            // Register callbacks
            register_keypress_callback(keypress_callback);
            register_menu_item_callback(menu_item_callback);

            let ico_file_c = U16CString::from_str(ico_icon).unwrap();
            let bmp_file_c = U16CString::from_str(bmp_icon).unwrap();

            // Initialize the windows
            let res = initialize(manager_ptr, ico_file_c.as_ptr(), bmp_file_c.as_ptr());
            if res != 1 {
                panic!("Can't initialize Windows context")
            }
        }

        manager
    }
}

impl super::Context for WindowsContext {
    fn eventloop(&self) {
        unsafe {
            eventloop();
        }
    }
}

// Native bridge code

extern fn keypress_callback(_self: *mut c_void, raw_buffer: *const i32, len: i32,
                            is_modifier: i32, key_code: i32) {
    unsafe {
        let _self = _self as *mut WindowsContext;

        if is_modifier == 0 {  // Char event
            // Convert the received buffer to a character
            let buffer = std::slice::from_raw_parts(raw_buffer, len as usize);
            let r = std::char::from_u32(buffer[0] as u32);

            // Send the char through the channel
            if let Some(c) = r {
                let event = Event::Key(KeyEvent::Char(c));
                (*_self).send_channel.send(event).unwrap();
            }
        }else{  // Modifier event
            let modifier: Option<KeyModifier> = match key_code {
                0x5B | 0x5C => Some(META),
                0x10 => Some(SHIFT),
                0x12 => Some(ALT),
                0x11 => Some(CTRL),
                0x08  => Some(BACKSPACE),
                _ => None,
            };

            if let Some(modifier) = modifier {
                let event = Event::Key(KeyEvent::Modifier(modifier));
                (*_self).send_channel.send(event).unwrap();
            }
        }
    }
}

extern fn menu_item_callback(_self: *mut c_void, items: *mut WindowsMenuItem, count: *mut i32) {
    unsafe {
        let _self = _self as *mut WindowsContext;

        let str = U16CString::from_str("Test").unwrap_or_default();
        let mut str_buff : [u16; 100] = [0; 100];
        std::ptr::copy(str.as_ptr(), str_buff.as_mut_ptr(), str.len());
        let item = WindowsMenuItem {
            item_id: 1,
            item_type: 1,
            item_name: str_buff,
        };

        let items = unsafe { std::slice::from_raw_parts_mut(items, 100) };

        items[0] = item;
        *count = 1;
    }
}
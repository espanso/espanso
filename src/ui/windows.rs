/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::bridge::windows::{close_notification, WindowsMenuItem, show_context_menu, cleanup_ui};
use widestring::U16CString;
use std::{thread, time};
use log::{debug};
use std::sync::Mutex;
use std::sync::Arc;
use crate::ui::{MenuItem, MenuItemType};
use std::path::Path;
use winrt_notification::{Duration, IconCrop, Sound, Toast};
use std::io;

pub struct WindowsUIManager {
    id: Arc<Mutex<i32>>
}

impl super::UIManager for WindowsUIManager {
    fn notify(&self, message: &str) {
        fn get_icon_path() -> io::Result<Box<Path>> {
            let path_buf = std::env::current_exe()?;
            let path_buf = path_buf.parent().unwrap().to_path_buf();
            let installed_ico = path_buf.join("icon.ico");
            let dev_ico = path_buf.parent().unwrap().parent().unwrap().join("packager/win/icon.ico");

            if installed_ico.is_file() {
                Ok(installed_ico.into_boxed_path())
            } else if dev_ico.is_file() {
                Ok(dev_ico.into_boxed_path())
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "icon.ico not found"))
            }
        }

        // Create and show a window notification
        let mut toast: Toast = Toast::new(Toast::POWERSHELL_APP_ID) // TODO: Use an ID assigned during installation.
            .title("Espanso")
            .text1(message)
            .duration(Duration::Short);
    
        if let Ok(p) = get_icon_path() {
            toast = toast
            .icon(
                &p,
                IconCrop::Circular,
                "espanso",
            );
        } else {
            toast = toast.text2(":(");
        }
        
        toast.show().expect("Unable to toast");

    }

    

    fn show_menu(&self, menu: Vec<MenuItem>) {
        let mut raw_menu = Vec::new();

        for item in menu.iter() {
            let text = U16CString::from_str(item.item_name.clone()).unwrap_or_default();
            let mut str_buff : [u16; 100] = [0; 100];
            unsafe {
                std::ptr::copy(text.as_ptr(), str_buff.as_mut_ptr(), text.len());
            }

            let menu_type = match item.item_type {
                MenuItemType::Button => {1},
                MenuItemType::Separator => {2},
            };

            let raw_item = WindowsMenuItem {
                item_id: item.item_id,
                item_type: menu_type,
                item_name: str_buff,
            };

            raw_menu.push(raw_item);
        }

        unsafe { show_context_menu(raw_menu.as_ptr(), raw_menu.len() as i32); }
    }

    fn cleanup(&self) {
        unsafe {
            cleanup_ui();
        }
    }
}

impl WindowsUIManager {
    pub fn new() -> WindowsUIManager {
        let id = Arc::new(Mutex::new(0));

        let manager = WindowsUIManager {
            id
        };

        manager
    }
}
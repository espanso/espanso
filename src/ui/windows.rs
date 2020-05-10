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

use crate::bridge::windows::{
    cleanup_ui, close_notification, show_context_menu, show_notification, WindowsMenuItem,
};
use crate::ui::{MenuItem, MenuItemType};
use log::debug;
use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time};
use widestring::U16CString;

pub struct WindowsUIManager {
    id: Arc<Mutex<i32>>,
}

impl super::UIManager for WindowsUIManager {
    fn notify(&self, message: &str) {
        self.notify_delay(message, 2000);
    }

    fn notify_delay(&self, message: &str, duration: i32) {
        let current_id: i32 = {
            let mut id = self.id.lock().unwrap();
            *id += 1;
            *id
        };

        let step = duration / 10;

        // Setup a timeout to close the notification
        let id = Arc::clone(&self.id);
        let _ = thread::Builder::new()
            .name("notification_thread".to_string())
            .spawn(move || {
                for _ in 1..10 {
                    let duration = time::Duration::from_millis(step as u64);
                    thread::sleep(duration);

                    let new_id = id.lock().unwrap();
                    if *new_id != current_id {
                        debug!("Cancelling notification close event with id {}", current_id);
                        return;
                    }
                }

                unsafe {
                    close_notification();
                }
            });

        // Create and show a window notification
        unsafe {
            let message = U16CString::from_str(message).unwrap();
            show_notification(message.as_ptr());
        }
    }

    fn show_menu(&self, menu: Vec<MenuItem>) {
        let mut raw_menu = Vec::new();

        for item in menu.iter() {
            let text = U16CString::from_str(item.item_name.clone()).unwrap_or_default();
            let mut str_buff: [u16; 100] = [0; 100];
            unsafe {
                std::ptr::copy(text.as_ptr(), str_buff.as_mut_ptr(), text.len());
            }

            let menu_type = match item.item_type {
                MenuItemType::Button => 1,
                MenuItemType::Separator => 2,
            };

            let raw_item = WindowsMenuItem {
                item_id: item.item_id,
                item_type: menu_type,
                item_name: str_buff,
            };

            raw_menu.push(raw_item);
        }

        unsafe {
            show_context_menu(raw_menu.as_ptr(), raw_menu.len() as i32);
        }
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

        let manager = WindowsUIManager { id };

        manager
    }
}

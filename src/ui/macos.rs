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

use crate::bridge::macos::{show_context_menu, MacMenuItem};
use crate::context;
use crate::ui::{MenuItem, MenuItemType};
use log::{debug, info, warn};
use std::ffi::CString;
use std::io::Cursor;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, io};

const NOTIFY_HELPER_BINARY: &'static [u8] = include_bytes!("../res/mac/EspansoNotifyHelper.zip");
const DEFAULT_NOTIFICATION_DELAY: f64 = 1.5;

pub struct MacUIManager {
    notify_helper_path: PathBuf,
}

impl super::UIManager for MacUIManager {
    fn notify(&self, message: &str) {
        let executable_path = self.notify_helper_path.join("Contents");
        let executable_path = executable_path.join("MacOS");
        let executable_path = executable_path.join("EspansoNotifyHelper");

        let res = Command::new(executable_path)
            .args(&["espanso", message, &DEFAULT_NOTIFICATION_DELAY.to_string()])
            .spawn();

        if let Err(e) = res {
            warn!("Error while dispatching Notify Helper {}", e)
        }
    }

    fn show_menu(&self, menu: Vec<MenuItem>) {
        let mut raw_menu = Vec::new();

        for item in menu.iter() {
            let text = CString::new(item.item_name.clone()).unwrap_or_default();
            let mut str_buff: [c_char; 100] = [0; 100];
            unsafe {
                std::ptr::copy(text.as_ptr(), str_buff.as_mut_ptr(), item.item_name.len());
            }

            let menu_type = match item.item_type {
                MenuItemType::Button => 1,
                MenuItemType::Separator => 2,
            };

            let raw_item = MacMenuItem {
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
        // Nothing to do here
    }
}

impl MacUIManager {
    pub fn new() -> MacUIManager {
        let notify_helper_path = MacUIManager::initialize_notify_helper();

        MacUIManager { notify_helper_path }
    }

    fn initialize_notify_helper() -> PathBuf {
        let espanso_dir = context::get_data_dir();

        info!(
            "Initializing EspansoNotifyHelper in {}",
            espanso_dir.as_path().display()
        );

        let espanso_target = espanso_dir.join("EspansoNotifyHelper.app");

        if espanso_target.exists() {
            info!("EspansoNotifyHelper already initialized, skipping.");
        } else {
            // Extract zip file
            let reader = Cursor::new(NOTIFY_HELPER_BINARY);

            let mut archive = zip::ZipArchive::new(reader).unwrap();

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).unwrap();
                let outpath = espanso_dir.join(file.sanitized_name());

                {
                    let comment = file.comment();
                    if !comment.is_empty() {
                        debug!("File {} comment: {}", i, comment);
                    }
                }

                if (&*file.name()).ends_with('/') {
                    debug!(
                        "File {} extracted to \"{}\"",
                        i,
                        outpath.as_path().display()
                    );
                    fs::create_dir_all(&outpath).unwrap();
                } else {
                    debug!(
                        "File {} extracted to \"{}\" ({} bytes)",
                        i,
                        outpath.as_path().display(),
                        file.size()
                    );
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            fs::create_dir_all(&p).unwrap();
                        }
                    }
                    let mut outfile = fs::File::create(&outpath).unwrap();
                    io::copy(&mut file, &mut outfile).unwrap();
                }

                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }

        espanso_target
    }
}

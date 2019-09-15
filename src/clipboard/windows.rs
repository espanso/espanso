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

use std::process::{Command, Stdio};
use std::io::{Write};

pub struct WindowsClipboardManager {

}

impl WindowsClipboardManager {
    pub fn new() -> WindowsClipboardManager {
        WindowsClipboardManager{}
    }
}

impl super::ClipboardManager for WindowsClipboardManager {
    fn get_clipboard(&self) -> Option<String>  {
        unimplemented!();
    }

    fn set_clipboard(&self, payload: &str) {
        unimplemented!();
    }
}
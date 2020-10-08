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
    get_clipboard, set_clipboard, set_clipboard_html, set_clipboard_image,
};
use std::{ffi::CString, path::Path};
use widestring::U16CString;

pub struct WindowsClipboardManager {}

impl WindowsClipboardManager {
    pub fn new() -> WindowsClipboardManager {
        WindowsClipboardManager {}
    }
}

impl super::ClipboardManager for WindowsClipboardManager {
    fn get_clipboard(&self) -> Option<String> {
        unsafe {
            let mut buffer: [u16; 2000] = [0; 2000];
            let res = get_clipboard(buffer.as_mut_ptr(), buffer.len() as i32);

            if res > 0 {
                let c_string = U16CString::from_ptr_str(buffer.as_ptr());

                let string = c_string.to_string_lossy();
                return Some((*string).to_owned());
            }
        }

        None
    }

    fn set_clipboard(&self, payload: &str) {
        unsafe {
            let payload_c = U16CString::from_str(payload).unwrap();
            set_clipboard(payload_c.as_ptr());
        }
    }

    fn set_clipboard_image(&self, image_path: &Path) {
        let path_string = image_path.to_string_lossy().into_owned();
        unsafe {
            let payload_c = U16CString::from_str(path_string).unwrap();
            set_clipboard_image(payload_c.as_ptr());
        }
    }

    fn set_clipboard_html(&self, html: &str) {
        // In order to set the HTML clipboard, we have to create a prefix with a specific format
        // For more information, look here:
        // https://docs.microsoft.com/en-us/windows/win32/dataxchg/html-clipboard-format
        // https://docs.microsoft.com/en-za/troubleshoot/cpp/add-html-code-clipboard
        let mut tokens = Vec::new();
        tokens.push("Version:0.9");
        tokens.push("StartHTML:<<STR*#>");
        tokens.push("EndHTML:<<END*#>");
        tokens.push("StartFragment:<<SFG#*>");
        tokens.push("EndFragment:<<EFG#*>");
        tokens.push("<html>");
        tokens.push("<body>");
        let content = format!("<!--StartFragment-->{}<!--EndFragment-->", html);
        tokens.push(&content);
        tokens.push("</body>");
        tokens.push("</html>");

        let mut render = tokens.join("\r\n");

        // Now replace the placeholders with the actual positions
        render = render.replace(
            "<<STR*#>",
            &format!("{:0>8}", render.find("<html>").unwrap_or_default()),
        );
        render = render.replace("<<END*#>", &format!("{:0>8}", render.len()));
        render = render.replace(
            "<<SFG#*>",
            &format!(
                "{:0>8}",
                render.find("<!--StartFragment-->").unwrap_or_default()
                    + "<!--StartFragment-->".len()
            ),
        );
        render = render.replace(
            "<<EFG#*>",
            &format!(
                "{:0>8}",
                render.find("<!--EndFragment-->").unwrap_or_default()
            ),
        );

        // Render the text fallback for those applications that don't support HTML clipboard
        let decorator = html2text::render::text_renderer::TrivialDecorator::new();
        let text_fallback =
            html2text::from_read_with_decorator(html.as_bytes(), 1000000, decorator);
        unsafe {
            let payload_c =
                CString::new(render).expect("unable to create CString for html content");
            let payload_fallback_c = U16CString::from_str(text_fallback).unwrap();
            set_clipboard_html(payload_c.as_ptr(), payload_fallback_c.as_ptr());
        }
    }
}

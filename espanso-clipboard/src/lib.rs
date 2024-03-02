/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
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

use std::path::Path;

use anyhow::Result;
use log::info;

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
mod x11;

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
mod wayland;

#[cfg(target_os = "macos")]
mod cocoa;

pub trait Clipboard {
    fn get_text(&self, options: &ClipboardOperationOptions) -> Option<String>;
    fn set_text(&self, text: &str, options: &ClipboardOperationOptions) -> Result<()>;
    fn set_image(&self, image_path: &Path, options: &ClipboardOperationOptions) -> Result<()>;
    fn set_html(
        &self,
        html: &str,
        fallback_text: Option<&str>,
        options: &ClipboardOperationOptions,
    ) -> Result<()>;
}

#[allow(dead_code)]
#[derive(Default)]
pub struct ClipboardOperationOptions {
    pub use_xclip_backend: bool,
}

#[allow(dead_code)]
pub struct ClipboardOptions {
    // Wayland-only
    // The number of milliseconds the wl-clipboard commands are allowed
    // to run before triggering a time-out event.
    wayland_command_timeout_ms: u64,
}

impl Default for ClipboardOptions {
    fn default() -> Self {
        Self {
            wayland_command_timeout_ms: 2000,
        }
    }
}

#[cfg(target_os = "windows")]
pub fn get_clipboard(_: ClipboardOptions) -> Result<Box<dyn Clipboard>> {
    info!("using Win32Clipboard");
    Ok(Box::new(win32::Win32Clipboard::new()?))
}

#[cfg(target_os = "macos")]
pub fn get_clipboard(_: ClipboardOptions) -> Result<Box<dyn Clipboard>> {
    info!("using CocoaClipboard");
    Ok(Box::new(cocoa::CocoaClipboard::new()?))
}

#[cfg(target_os = "linux")]
#[cfg(not(feature = "wayland"))]
pub fn get_clipboard(_: ClipboardOptions) -> Result<Box<dyn Clipboard>> {
    info!("using X11Clipboard");
    Ok(Box::new(x11::X11Clipboard::new()?))
}

#[cfg(target_os = "linux")]
#[cfg(feature = "wayland")]
pub fn get_clipboard(options: ClipboardOptions) -> Result<Box<dyn Clipboard>> {
    // TODO: On some Wayland compositors (currently sway), the "wlr-data-control" protocol
    // could enable the use of a much more efficient implementation relying on the "wl-clipboard-rs" crate.
    // Useful links: https://github.com/YaLTeR/wl-clipboard-rs/issues/8
    //
    // We could even decide the correct implementation at runtime by checking if the
    // required protocol is available, if so use the efficient implementation
    // instead of the fallback one, which calls the wl-copy and wl-paste binaries, and is thus
    // less efficient

    info!("using WaylandFallbackClipboard");
    Ok(Box::new(wayland::fallback::WaylandFallbackClipboard::new(
        options,
    )?))
}

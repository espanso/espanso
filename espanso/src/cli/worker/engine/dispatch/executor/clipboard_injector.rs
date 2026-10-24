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

use std::{convert::TryInto, path::PathBuf};

use espanso_clipboard::{Clipboard, ClipboardOperationOptions};
use espanso_inject::{keys::Key, InjectionOptions, Injector};
use log::error;

use espanso_engine::{
    dispatch::HtmlInjector,
    dispatch::{ImageInjector, TextInjector},
};

pub trait ClipboardParamsProvider {
    fn get(&self) -> ClipboardParams;
}

pub struct ClipboardParams {
    pub pre_paste_delay: usize,
    pub paste_shortcut_event_delay: usize,
    pub paste_shortcut: Option<String>,
    pub disable_x11_fast_inject: bool,
    pub restore_clipboard: bool,
    pub restore_clipboard_delay: usize,
    pub x11_use_xclip_backend: bool,
    pub x11_use_xdotool_backend: bool,
}

pub struct ClipboardInjectorAdapter<'a> {
    injector: &'a dyn Injector,
    clipboard: &'a dyn Clipboard,
    params_provider: &'a dyn ClipboardParamsProvider,
}

impl<'a> ClipboardInjectorAdapter<'a> {
    pub fn new(
        injector: &'a dyn Injector,
        clipboard: &'a dyn Clipboard,
        params_provider: &'a dyn ClipboardParamsProvider,
    ) -> Self {
        Self {
            injector,
            clipboard,
            params_provider,
        }
    }

    fn paste_combination(&self) -> Vec<Key> {
        let params = self.params_provider.get();

        let mut custom_combination = None;
        if let Some(custom_shortcut) = params.paste_shortcut {
            if let Some(combination) = parse_combination(&custom_shortcut) {
                custom_combination = Some(combination);
            } else {
                error!("'{custom_shortcut}' is not a valid paste shortcut");
            }
        }

        if let Some(custom_combination) = custom_combination {
            custom_combination
        } else if cfg!(target_os = "macos") {
            vec![Key::Meta, Key::V]
        } else if cfg!(target_os = "linux") && cfg!(feature = "wayland") {
            // Because on Wayland we currently don't have app-specific configs (and therefore no patches)
            // we switch to the more supported SHIFT+INSERT combination
            // See: https://github.com/espanso/espanso/issues/899
            vec![Key::Shift, Key::Insert]
        } else {
            vec![Key::Control, Key::V]
        }
    }

    fn send_paste_combination(&self, combination: &[Key]) -> anyhow::Result<()> {
        let params = self.params_provider.get();

        std::thread::sleep(std::time::Duration::from_millis(
            params.pre_paste_delay.try_into().unwrap(),
        ));

        self.injector.send_key_combination(
            combination,
            InjectionOptions {
                delay: params.paste_shortcut_event_delay as i32,
                disable_fast_inject: params.disable_x11_fast_inject,
                x11_use_xdotool_fallback: params.x11_use_xdotool_backend,
                ..Default::default()
            },
        )?;

        Ok(())
    }

    fn restore_clipboard_guard(&self, include_primary: bool) -> Option<ClipboardRestoreGuard<'a>> {
        let params = self.params_provider.get();

        if params.restore_clipboard {
            Some(ClipboardRestoreGuard::lock(
                self.clipboard,
                params.restore_clipboard_delay.try_into().unwrap(),
                self.get_operation_options(),
                include_primary,
            ))
        } else {
            None
        }
    }

    fn get_operation_options(&self) -> ClipboardOperationOptions {
        let params = self.params_provider.get();
        ClipboardOperationOptions {
            use_xclip_backend: params.x11_use_xclip_backend,
            primary: false,
        }
    }

    // Mirror the injected text into the primary selection.
    // This is best-effort: apps pasting from the clipboard still work if it fails
    // (e.g. on compositors without primary selection support). In that case the
    // primary selection is also left alone when restoring.
    fn set_primary_text(&self, text: &str, guard: &mut Option<ClipboardRestoreGuard>) {
        if let Err(error) = self
            .clipboard
            .set_text(text, &primary_options(&self.get_operation_options()))
        {
            error!("unable to set primary selection: {error}");
            if let Some(guard) = guard {
                guard.skip_primary_restore();
            }
        }
    }
}

impl TextInjector for ClipboardInjectorAdapter<'_> {
    fn name(&self) -> &'static str {
        "clipboard"
    }

    fn inject_text(&self, text: &str) -> anyhow::Result<()> {
        let combination = self.paste_combination();
        let use_primary = uses_primary_selection(&combination);
        let mut guard = self.restore_clipboard_guard(use_primary);

        self.clipboard
            .set_text(text, &self.get_operation_options())?;
        if use_primary {
            self.set_primary_text(text, &mut guard);
        }

        self.send_paste_combination(&combination)?;

        Ok(())
    }
}

impl HtmlInjector for ClipboardInjectorAdapter<'_> {
    fn inject_html(&self, html: &str, fallback_text: &str) -> anyhow::Result<()> {
        let combination = self.paste_combination();
        let use_primary = uses_primary_selection(&combination);
        let mut guard = self.restore_clipboard_guard(use_primary);

        self.clipboard
            .set_html(html, Some(fallback_text), &self.get_operation_options())?;
        if use_primary {
            self.set_primary_text(fallback_text, &mut guard);
        }

        self.send_paste_combination(&combination)?;

        Ok(())
    }
}

impl ImageInjector for ClipboardInjectorAdapter<'_> {
    fn inject_image(&self, image_path: &str) -> anyhow::Result<()> {
        let path = PathBuf::from(image_path);
        if !path.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "image can't be found in the given path",
            )
            .into());
        }

        let combination = self.paste_combination();
        let _guard = self.restore_clipboard_guard(false);

        self.clipboard
            .set_image(&path, &self.get_operation_options())?;

        self.send_paste_combination(&combination)?;

        Ok(())
    }
}

struct ClipboardRestoreGuard<'a> {
    clipboard: &'a dyn Clipboard,
    content: Option<String>,
    // None if the primary selection is not involved in the paste.
    // An empty string means the primary selection was empty.
    primary_content: Option<String>,
    restore_delay: u64,
    clipboard_operation_options: ClipboardOperationOptions,
}

impl<'a> ClipboardRestoreGuard<'a> {
    pub fn lock(
        clipboard: &'a dyn Clipboard,
        restore_delay: u64,
        clipboard_operation_options: ClipboardOperationOptions,
        include_primary: bool,
    ) -> Self {
        let clipboard_content = clipboard.get_text(&clipboard_operation_options);
        let primary_content = if include_primary {
            // Reading also fails if the primary selection is empty, so any read error
            // is treated as empty and the primary selection gets cleared on restore
            Some(
                clipboard
                    .get_text(&primary_options(&clipboard_operation_options))
                    .unwrap_or_default(),
            )
        } else {
            None
        };

        Self {
            clipboard,
            content: clipboard_content,
            primary_content,
            restore_delay,
            clipboard_operation_options,
        }
    }

    fn skip_primary_restore(&mut self) {
        self.primary_content = None;
    }
}

impl Drop for ClipboardRestoreGuard<'_> {
    fn drop(&mut self) {
        let content = self.content.take();
        let primary_content = self.primary_content.take();
        if content.is_none() && primary_content.is_none() {
            return;
        }

        // Sometimes an expansion gets overwritten before pasting by the previous content
        // A delay is needed to mitigate the problem
        std::thread::sleep(std::time::Duration::from_millis(self.restore_delay));

        if let Some(content) = content {
            if let Err(error) = self
                .clipboard
                .set_text(&content, &self.clipboard_operation_options)
            {
                error!("unable to restore clipboard content after expansion: {error}");
            }
        }

        if let Some(primary_content) = primary_content {
            if let Err(error) = self.clipboard.set_text(
                &primary_content,
                &primary_options(&self.clipboard_operation_options),
            ) {
                error!("unable to restore primary selection after expansion: {error}");
            }
        }
    }
}

fn parse_combination(combination: &str) -> Option<Vec<Key>> {
    let tokens = combination.split('+');
    let mut keys: Vec<Key> = Vec::new();
    for token in tokens {
        keys.push(Key::parse(token)?);
    }

    Some(keys)
}

// Some terminals (e.g. VTE based ones) paste the primary selection instead of the
// clipboard on SHIFT+INSERT, so on Wayland the primary selection needs to be handled too.
fn uses_primary_selection(combination: &[Key]) -> bool {
    cfg!(all(target_os = "linux", feature = "wayland"))
        && matches!(combination, [Key::Shift, Key::Insert])
}

fn primary_options(options: &ClipboardOperationOptions) -> ClipboardOperationOptions {
    ClipboardOperationOptions {
        primary: true,
        ..*options
    }
}

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

use std::sync::Arc;

use espanso_config::config::{AppProperties, Config, ConfigStore};

mod config_store;
mod patches;

pub fn patch_store(store: Box<dyn ConfigStore>) -> Box<dyn ConfigStore> {
    Box::new(config_store::PatchedConfigStore::from_store(store))
}

fn get_builtin_patches() -> Vec<PatchDefinition> {
    #[cfg(target_os = "windows")]
    return vec![
        patches::win::brave::patch(),
        patches::win::onenote_for_windows_10::patch(),
        patches::win::vscode_win::patch(),
    ];

    #[cfg(target_os = "macos")]
    return vec![];

    #[cfg(target_os = "linux")]
    return vec![
        patches::linux::alacritty_terminal_x11::patch(),
        patches::linux::emacs_x11::patch(),
        patches::linux::gedit_x11::patch(),
        patches::linux::generic_terminal_x11::patch(),
        patches::linux::kitty_terminal_x11::patch(),
        patches::linux::konsole_terminal_x11::patch(),
        patches::linux::libreoffice_writer_x11::patch(),
        patches::linux::simple_terminal_x11::patch(),
        patches::linux::simple_terminal_2_x11::patch(),
        patches::linux::terminator_terminal_x11::patch(),
        patches::linux::termite_terminal_x11::patch(),
        patches::linux::thunderbird_x11::patch(),
        patches::linux::tilix_terminal_x11::patch(),
        patches::linux::urxvt_terminal_x11::patch(),
        patches::linux::xterm_terminal_x11::patch(),
        patches::linux::yakuake_terminal_x11::patch(),
        patches::linux::virtualbox_x11::patch(),
    ];
}

pub struct PatchDefinition {
    pub name: &'static str,
    pub is_enabled: fn() -> bool,
    pub should_patch: fn(app: &AppProperties) -> bool,
    pub apply: fn(config: Arc<dyn Config>, name: &str) -> Arc<dyn Config>,
}

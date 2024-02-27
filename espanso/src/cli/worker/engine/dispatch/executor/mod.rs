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

pub mod clipboard_injector;
pub mod context_menu;
pub mod event_injector;
pub mod icon;
pub mod key_injector;
pub mod secure_input;
pub mod text_ui;

pub trait InjectParamsProvider {
    fn get(&self) -> InjectParams;
}

pub struct InjectParams {
    pub inject_delay: Option<usize>,
    pub key_delay: Option<usize>,
    pub disable_x11_fast_inject: bool,
    pub evdev_modifier_delay: Option<usize>,
    pub x11_use_xdotool_backend: bool,
}

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

use std::time::Duration;

use espanso_config::config::Config;
use espanso_engine::{
    event::input::{Key, Variant},
    process::DisableOptions,
};

pub fn extract_disable_options(config: &dyn Config) -> DisableOptions {
    let (toggle_key, variant) = match config.toggle_key() {
        Some(key) => match key {
            espanso_config::config::ToggleKey::Ctrl => (Some(Key::Control), None),
            espanso_config::config::ToggleKey::Meta => (Some(Key::Meta), None),
            espanso_config::config::ToggleKey::Alt => (Some(Key::Alt), None),
            espanso_config::config::ToggleKey::Shift => (Some(Key::Shift), None),
            espanso_config::config::ToggleKey::RightCtrl => {
                (Some(Key::Control), Some(Variant::Right))
            }
            espanso_config::config::ToggleKey::RightAlt => (Some(Key::Alt), Some(Variant::Right)),
            espanso_config::config::ToggleKey::RightShift => {
                (Some(Key::Shift), Some(Variant::Right))
            }
            espanso_config::config::ToggleKey::RightMeta => (Some(Key::Meta), Some(Variant::Right)),
            espanso_config::config::ToggleKey::LeftCtrl => {
                (Some(Key::Control), Some(Variant::Left))
            }
            espanso_config::config::ToggleKey::LeftAlt => (Some(Key::Alt), Some(Variant::Left)),
            espanso_config::config::ToggleKey::LeftShift => (Some(Key::Shift), Some(Variant::Left)),
            espanso_config::config::ToggleKey::LeftMeta => (Some(Key::Meta), Some(Variant::Left)),
        },
        None => (None, None),
    };

    DisableOptions {
        toggle_key,
        toggle_key_variant: variant,
        toggle_key_maximum_window: Duration::from_millis(1000),
    }
}

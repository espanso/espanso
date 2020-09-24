/*
 * This file is part of espanso.
 *
 * Copyright (C) 2020 Federico Terzi
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

use crate::{config::Configs, extension::ExtensionResult, ui::modulo::ModuloManager};
use log::{error, warn};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;

pub struct FormExtension {
    manager: ModuloManager,
}

impl FormExtension {
    pub fn new(config: &Configs) -> FormExtension {
        let manager = ModuloManager::new(config);
        FormExtension { manager }
    }
}

impl super::Extension for FormExtension {
    fn name(&self) -> String {
        "form".to_owned()
    }

    fn calculate(
        &self,
        params: &Mapping,
        _: &Vec<String>,
        _: &HashMap<String, ExtensionResult>,
    ) -> Option<ExtensionResult> {
        let layout = params.get(&Value::from("layout"));
        let layout = if let Some(value) = layout {
            value.as_str().unwrap_or_default().to_string()
        } else {
            error!("invoking form extension without specifying a layout");
            return None;
        };

        let mut form_config = Mapping::new();
        form_config.insert(Value::from("title"), Value::from("espanso"));
        form_config.insert(Value::from("layout"), Value::from(layout));

        if let Some(fields) = params.get(&Value::from("fields")) {
            form_config.insert(Value::from("fields"), fields.clone());
        }

        if let Some(icon_path) = crate::context::get_icon_path() {
            form_config.insert(
                Value::from("icon"),
                Value::from(icon_path.to_string_lossy().to_string()),
            );
        }

        let serialized_config: String =
            serde_yaml::to_string(&form_config).expect("unable to serialize form config");

        let output = self
            .manager
            .invoke(&["form", "-i", "-"], &serialized_config);

        // On macOS and Windows, after the form closes we have to wait until the user releases the modifier keys
        on_form_close();

        if let Some(output) = output {
            let json: Result<HashMap<String, String>, _> = serde_json::from_str(&output);
            match json {
                Ok(json) => {
                    return Some(ExtensionResult::Multiple(json));
                }
                Err(error) => {
                    error!("modulo json parsing error: {}", error);
                    return None;
                }
            }
        } else {
            error!("modulo form didn't return any output");
            return None;
        }
    }
}

#[cfg(target_os = "linux")]
fn on_form_close() {
    // NOOP on Linux
}

#[cfg(target_os = "windows")]
fn on_form_close() {
    let released = crate::keyboard::windows::wait_for_modifiers_release();
    if !released {
        warn!("Wait for modifiers release timed out! Please after closing the form, release your modifiers keys (CTRL, CMD, ALT, SHIFT)");
    }
}

#[cfg(target_os = "macos")]
fn on_form_close() {
    let released = crate::keyboard::macos::wait_for_modifiers_release();
    if !released {
        warn!("Wait for modifiers release timed out! Please after closing the form, release your modifiers keys (CTRL, CMD, ALT, SHIFT)");
    }
}

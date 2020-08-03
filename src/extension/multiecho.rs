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

use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use crate::extension::ExtensionResult;

pub struct MultiEchoExtension {}

impl MultiEchoExtension {
    pub fn new() -> MultiEchoExtension {
        MultiEchoExtension {}
    }
}

impl super::Extension for MultiEchoExtension {
    fn name(&self) -> String {
        "multiecho".to_owned()
    }

    fn calculate(&self, params: &Mapping, _: &Vec<String>, _: &HashMap<String, ExtensionResult>) -> Option<ExtensionResult> {
        let mut output: HashMap<String, String> = HashMap::new();
        for (key, value) in params.iter() {
            if let Some(key) = key.as_str() {
                if let Some(value) = value.as_str() {
                    output.insert(key.to_owned(), value.to_owned());
                }
            }
        }
        Some(ExtensionResult::Multiple(output))
    }
}

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

pub struct VarDummyExtension {}

impl VarDummyExtension {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::Extension for VarDummyExtension {
    fn name(&self) -> String {
        "vardummy".to_owned()
    }

    fn calculate(&self, params: &Mapping, _: &Vec<String>, vars: &HashMap<String, ExtensionResult>) -> Option<ExtensionResult> {
        let target = params.get(&Value::from("target"));

        if let Some(target) = target {
            let value = vars.get(target.as_str().unwrap_or_default());
            Some(value.unwrap().clone())
        } else {
            None
        }
    }
}

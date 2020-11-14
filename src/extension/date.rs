/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2020 Federico Terzi
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

use crate::extension::ExtensionResult;
use chrono::{DateTime, Duration, Local};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;

use super::ExtensionOut;

pub struct DateExtension {}

impl DateExtension {
    pub fn new() -> DateExtension {
        DateExtension {}
    }
}

impl super::Extension for DateExtension {
    fn name(&self) -> String {
        String::from("date")
    }

    fn calculate(
        &self,
        params: &Mapping,
        _: &Vec<String>,
        _: &HashMap<String, ExtensionResult>,
    ) -> ExtensionOut {
        let mut now: DateTime<Local> = Local::now();

        // Compute the given offset
        let offset = params.get(&Value::from("offset"));
        if let Some(offset) = offset {
            let seconds = offset.as_i64().unwrap_or_else(|| 0);
            let offset = Duration::seconds(seconds);
            now = now + offset;
        }

        let format = params.get(&Value::from("format"));

        let date = if let Some(format) = format {
            now.format(format.as_str().unwrap()).to_string()
        } else {
            now.to_rfc2822()
        };

        Ok(Some(ExtensionResult::Single(date)))
    }
}

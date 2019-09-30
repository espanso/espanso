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

use serde_yaml::{Mapping, Value};
use chrono::{DateTime, Local};

pub struct DateExtension {}

impl DateExtension {
    pub fn new() -> DateExtension {
        DateExtension{}
    }
}

impl super::Extension for DateExtension {
    fn name(&self) -> String {
        String::from("date")
    }

    fn calculate(&self, params: &Mapping) -> Option<String> {
        let now: DateTime<Local> = Local::now();

        let format = params.get(&Value::from("format"));

        let date = if let Some(format) = format {
            now.format(format.as_str().unwrap()).to_string()
        }else{
            now.to_rfc2822()
        };

        Some(date)
    }
}
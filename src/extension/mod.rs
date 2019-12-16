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

use serde_yaml::Mapping;

mod date;
mod shell;
mod script;
mod random;

pub trait Extension {
    fn name(&self) -> String;
    fn calculate(&self, params: &Mapping) -> Option<String>;
}

pub fn get_extensions() -> Vec<Box<dyn Extension>> {
    vec![
        Box::new(date::DateExtension::new()),
        Box::new(shell::ShellExtension::new()),
        Box::new(script::ScriptExtension::new()),
        Box::new(random::RandomExtension::new()),
    ]
}
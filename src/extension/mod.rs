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

use crate::{clipboard::ClipboardManager, config::Configs};
use serde_yaml::Mapping;
use std::collections::HashMap;

mod clipboard;
mod date;
pub mod dummy;
mod form;
pub mod multiecho;
mod random;
mod script;
mod shell;
mod utils;
pub mod vardummy;

#[derive(Clone, Debug, PartialEq)]
pub enum ExtensionResult {
    Single(String),
    Multiple(HashMap<String, String>),
}

pub trait Extension {
    fn name(&self) -> String;
    fn calculate(
        &self,
        params: &Mapping,
        args: &Vec<String>,
        current_vars: &HashMap<String, ExtensionResult>,
    ) -> Option<ExtensionResult>;
}

pub fn get_extensions(
    config: &Configs,
    clipboard_manager: Box<dyn ClipboardManager>,
) -> Vec<Box<dyn Extension>> {
    vec![
        Box::new(date::DateExtension::new()),
        Box::new(shell::ShellExtension::new()),
        Box::new(script::ScriptExtension::new()),
        Box::new(random::RandomExtension::new()),
        Box::new(multiecho::MultiEchoExtension::new()),
        Box::new(dummy::DummyExtension::new("dummy")),
        Box::new(dummy::DummyExtension::new("echo")),
        Box::new(clipboard::ClipboardExtension::new(clipboard_manager)),
        Box::new(form::FormExtension::new(config)),
    ]
}

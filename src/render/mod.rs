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

use std::path::PathBuf;
use crate::matcher::{Match};
use crate::config::Configs;

pub(crate) mod default;
pub(crate) mod utils;

pub trait Renderer {
    // Render a match output
    fn render_match(&self, m: &Match, trigger_offset: usize, config: &Configs, args: Vec<String>) -> RenderResult;

    // Render a passive expansion text
    fn render_passive(&self, text: &str, config: &Configs) -> RenderResult;
}

pub enum RenderResult {
    Text(String),
    Image(PathBuf),
    Error
}
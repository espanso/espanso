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

use std::path::Path;
use anyhow::Result;

use crate::{config::ConfigStore, matches::store::MatchStore};

mod config;
mod model;

pub fn load(base_dir: &Path) -> Result<(Box<dyn ConfigStore>, Box<dyn MatchStore>)> {
  // TODO: load legacy config set and then convert it to the new format 

  todo!()
}
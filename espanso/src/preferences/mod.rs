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

mod default;

pub trait Preferences: Send + Sync {
  fn has_completed_wizard(&self) -> bool;
  fn set_completed_wizard(&self, value: bool);
}

pub fn get_default(runtime_dir: &Path) -> Result<impl Preferences> {
  let kvs = espanso_kvs::get_persistent(runtime_dir)?;
  default::DefaultPreferences::new(runtime_dir, kvs)
}
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

use anyhow::Result;
use std::path::Path;

mod default;

pub trait Preferences: Send + Sync + Clone {
  fn has_completed_wizard(&self) -> bool;
  fn set_completed_wizard(&self, value: bool);

  fn has_displayed_welcome(&self) -> bool;
  fn set_has_displayed_welcome(&self, value: bool);

  fn should_display_troubleshoot_for_non_fatal_errors(&self) -> bool;
  fn set_should_display_troubleshoot_for_non_fatal_errors(&self, value: bool);

  fn has_selected_auto_start_option(&self) -> bool;
  fn set_has_selected_auto_start_option(&self, value: bool);
}

pub fn get_default(runtime_dir: &Path) -> Result<impl Preferences> {
  let kvs = espanso_kvs::get_persistent(runtime_dir)?;
  default::DefaultPreferences::new(kvs)
}

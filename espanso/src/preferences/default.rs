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

use espanso_kvs::KVS;
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

use anyhow::Result;

use super::Preferences;

const HAS_COMPLETED_WIZARD_KEY: &str = "has_completed_wizard";

#[derive(Clone)]
pub struct DefaultPreferences<KVSType: KVS> {
  kvs: KVSType,
}

impl<KVSType: KVS> DefaultPreferences<KVSType> {
  pub fn new(runtime_dir: &Path, kvs: KVSType) -> Result<Self> {
    Ok(Self { kvs })
  }

  fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
    let value = self
      .kvs
      .get(key)
      .expect(&format!("unable to read preference for key {}", key));
    value
  }

  fn set<T: Serialize>(&self, key: &str, value: T) {
    self
      .kvs
      .set(key, value)
      .expect(&format!("unable to write preference for key {}", key))
  }
}

impl<KVSType: KVS> Preferences for DefaultPreferences<KVSType> {
  fn has_completed_wizard(&self) -> bool {
    self.get(HAS_COMPLETED_WIZARD_KEY).unwrap_or(false)
  }

  fn set_completed_wizard(&self, value: bool) {
    self.set(HAS_COMPLETED_WIZARD_KEY, value);
  }
}

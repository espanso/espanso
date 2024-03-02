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

use anyhow::Result;

use super::Preferences;

const HAS_COMPLETED_WIZARD_KEY: &str = "has_completed_wizard";
const HAS_DISPLAYED_WELCOME_KEY: &str = "has_displayed_welcome";
const SHOULD_DISPLAY_TROUBLESHOOT_FOR_NON_FATAL_ERRORS: &str =
  "should_display_troubleshoot_for_non_fatal_errors";
const HAS_SELECTED_AUTO_START_OPTION: &str = "has_selected_auto_start_option";

#[derive(Clone)]
pub struct DefaultPreferences<KVSType: KVS> {
  kvs: KVSType,
}

impl<KVSType: KVS> DefaultPreferences<KVSType> {
  pub fn new(kvs: KVSType) -> Result<Self> {
    Ok(Self { kvs })
  }

  fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
    self
      .kvs
      .get(key)
      .unwrap_or_else(|_| panic!("unable to read preference for key {key}"))
  }

  fn set<T: Serialize>(&self, key: &str, value: T) {
    self
      .kvs
      .set(key, value)
      .unwrap_or_else(|_| panic!("unable to write preference for key {key}"));
  }
}

impl<KVSType: KVS> Preferences for DefaultPreferences<KVSType> {
  fn has_completed_wizard(&self) -> bool {
    self.get(HAS_COMPLETED_WIZARD_KEY).unwrap_or(false)
  }

  fn set_completed_wizard(&self, value: bool) {
    self.set(HAS_COMPLETED_WIZARD_KEY, value);
  }

  fn has_displayed_welcome(&self) -> bool {
    self.get(HAS_DISPLAYED_WELCOME_KEY).unwrap_or(false)
  }

  fn set_has_displayed_welcome(&self, value: bool) {
    self.set(HAS_DISPLAYED_WELCOME_KEY, value);
  }

  fn should_display_troubleshoot_for_non_fatal_errors(&self) -> bool {
    self
      .get(SHOULD_DISPLAY_TROUBLESHOOT_FOR_NON_FATAL_ERRORS)
      .unwrap_or(true)
  }

  fn set_should_display_troubleshoot_for_non_fatal_errors(&self, value: bool) {
    self.set(SHOULD_DISPLAY_TROUBLESHOOT_FOR_NON_FATAL_ERRORS, value);
  }

  fn has_selected_auto_start_option(&self) -> bool {
    self.get(HAS_SELECTED_AUTO_START_OPTION).unwrap_or(false)
  }

  fn set_has_selected_auto_start_option(&self, value: bool) {
    self.set(HAS_SELECTED_AUTO_START_OPTION, value);
  }
}

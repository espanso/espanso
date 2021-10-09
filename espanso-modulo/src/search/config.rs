/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

use serde::{Deserialize, Serialize};

fn default_title() -> String {
  "espanso".to_owned()
}

fn default_icon() -> Option<String> {
  None
}

fn default_items() -> Vec<SearchItem> {
  Vec::new()
}

fn default_algorithm() -> String {
  "ikey".to_owned()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
  #[serde(default = "default_title")]
  pub title: String,

  #[serde(default = "default_icon")]
  pub icon: Option<String>,

  #[serde(default = "default_items")]
  pub items: Vec<SearchItem>,

  #[serde(default = "default_algorithm")]
  pub algorithm: String,

  #[serde(default)]
  pub hint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchItem {
  pub id: String,
  pub label: String,
  pub trigger: Option<String>,

  #[serde(default)]
  pub is_builtin: bool,
}

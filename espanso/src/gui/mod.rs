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

use std::{collections::HashMap, path::Path};

use anyhow::Result;

pub mod modulo;

pub trait SearchUI {
  fn show(&self, items: &[SearchItem], hint: Option<&str>) -> Result<Option<String>>;
}

#[derive(Debug)]
pub struct SearchItem {
  pub id: String,
  pub label: String,
  pub tag: Option<String>,
  pub additional_search_terms: Vec<String>,
  pub is_builtin: bool,
}

pub trait FormUI {
  fn show(
    &self,
    layout: &str,
    fields: &HashMap<String, FormField>,
  ) -> Result<Option<HashMap<String, String>>>;
}

#[derive(Debug)]
pub enum FormField {
  Text {
    default: Option<String>,
    multiline: bool,
  },
  Choice {
    default: Option<String>,
    values: Vec<String>,
  },
  List {
    default: Option<String>,
    values: Vec<String>,
  },
}

pub trait TextUI {
  fn show_text(&self, title: &str, text: &str) -> Result<()>;
  fn show_file(&self, title: &str, path: &Path) -> Result<()>;
}

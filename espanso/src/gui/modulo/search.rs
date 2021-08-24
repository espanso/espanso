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

use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap};

use crate::gui::{SearchItem, SearchUI};

use super::manager::ModuloManager;

pub struct ModuloSearchUI<'a> {
  manager: &'a ModuloManager,
}

impl<'a> ModuloSearchUI<'a> {
  pub fn new(manager: &'a ModuloManager) -> Self {
    Self {
      manager,
    }
  }
}

impl<'a> SearchUI for ModuloSearchUI<'a> {
  fn show(&self, items: &[SearchItem], hint: Option<&str>) -> anyhow::Result<Option<String>> {
    let modulo_config = ModuloSearchConfig {
      title: "espanso",
      hint,
      items: convert_items(&items),
    };

    let json_config = serde_json::to_string(&modulo_config)?;
    let output = self
      .manager
      .invoke(&["search", "-j", "-i", "-"], &json_config)?;
    let json: Result<HashMap<String, Value>, _> = serde_json::from_str(&output);
    match json {
      Ok(json) => {
        if let Some(Value::String(selected_id)) = json.get("selected") {
          return Ok(Some(selected_id.clone()));
        } else {
          return Ok(None);
        }
      }
      Err(error) => {
        return Err(error.into());
      }
    }
  }
}

#[derive(Debug, Serialize)]
struct ModuloSearchConfig<'a> {
  title: &'a str,
  hint: Option<&'a str>,
  items: Vec<ModuloSearchItemConfig<'a>>,
}

#[derive(Debug, Serialize)]
struct ModuloSearchItemConfig<'a> {
  id: &'a str,
  label: &'a str,
  trigger: Option<&'a str>,
  is_builtin: bool,
}

// TODO: test
fn convert_items<'a>(items: &'a [SearchItem]) -> Vec<ModuloSearchItemConfig<'a>> {
  items.iter().map(|item| ModuloSearchItemConfig {
    id: &item.id,
    label: &item.label, 
    trigger: item.tag.as_deref(), 
    is_builtin: item.is_builtin,
  }).collect()
}

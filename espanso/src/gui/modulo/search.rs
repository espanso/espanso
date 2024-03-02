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
use std::{collections::HashMap, convert::TryInto};

use crate::gui::{SearchItem, SearchUI};

use super::manager::ModuloManager;

pub trait ModuloSearchUIOptionProvider {
  fn get_post_search_delay(&self) -> usize;
}

pub struct ModuloSearchUI<'a> {
  manager: &'a ModuloManager,
  option_provider: &'a dyn ModuloSearchUIOptionProvider,
}

impl<'a> ModuloSearchUI<'a> {
  pub fn new(
    manager: &'a ModuloManager,
    option_provider: &'a dyn ModuloSearchUIOptionProvider,
  ) -> Self {
    Self {
      manager,
      option_provider,
    }
  }
}

impl<'a> SearchUI for ModuloSearchUI<'a> {
  fn show(&self, items: &[SearchItem], hint: Option<&str>) -> anyhow::Result<Option<String>> {
    let modulo_config = ModuloSearchConfig {
      title: "espanso",
      hint,
      items: convert_items(items),
    };

    let json_config = serde_json::to_string(&modulo_config)?;
    let output = self
      .manager
      .invoke(&["search", "-j", "-i", "-"], &json_config)?;
    let json: Result<HashMap<String, Value>, _> = serde_json::from_str(&output);
    let result = match json {
      Ok(json) => {
        if let Some(Value::String(selected_id)) = json.get("selected") {
          Ok(Some(selected_id.clone()))
        } else {
          Ok(None)
        }
      }
      Err(error) => Err(error.into()),
    };

    let post_search_delay = self.option_provider.get_post_search_delay();
    if post_search_delay > 0 {
      std::thread::sleep(std::time::Duration::from_millis(
        post_search_delay.try_into().unwrap(),
      ));
    }

    result
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
  search_terms: Vec<&'a str>,
  is_builtin: bool,
}

// TODO: test
fn convert_items(items: &[SearchItem]) -> Vec<ModuloSearchItemConfig> {
  items
    .iter()
    .map(|item| ModuloSearchItemConfig {
      id: &item.id,
      label: &item.label,
      trigger: item.tag.as_deref(),
      search_terms: if item.additional_search_terms.is_empty() {
        vec![]
      } else {
        item
          .additional_search_terms
          .iter()
          .map(String::as_str)
          .collect()
      },
      is_builtin: item.is_builtin,
    })
    .collect()
}

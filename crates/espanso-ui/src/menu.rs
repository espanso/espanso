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
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Menu {
  pub items: Vec<MenuItem>,
}

impl Menu {
  pub fn to_json(&self) -> Result<String> {
    Ok(serde_json::to_string(&self.items)?)
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum MenuItem {
  Simple(SimpleMenuItem),
  Sub(SubMenuItem),
  Separator,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleMenuItem {
  pub id: u32,
  pub label: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubMenuItem {
  pub label: String,
  pub items: Vec<MenuItem>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_context_menu_serializes_correctly() {
    let menu = Menu {
      items: vec![
        MenuItem::Simple(SimpleMenuItem {
          id: 0,
          label: "Open".to_string(),
        }),
        MenuItem::Separator,
        MenuItem::Sub(SubMenuItem {
          label: "Sub".to_string(),
          items: vec![
            MenuItem::Simple(SimpleMenuItem {
              label: "Sub 1".to_string(),
              id: 1,
            }),
            MenuItem::Simple(SimpleMenuItem {
              label: "Sub 2".to_string(),
              id: 2,
            }),
          ],
        }),
      ],
    };

    assert_eq!(
      menu.to_json().unwrap(),
      r#"[{"type":"simple","id":0,"label":"Open"},{"type":"separator"},{"type":"sub","label":"Sub","items":[{"type":"simple","id":1,"label":"Sub 1"},{"type":"simple","id":2,"label":"Sub 2"}]}]"#
    );
  }
}

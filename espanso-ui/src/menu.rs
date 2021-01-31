use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug)]
pub struct Menu {
  items: Vec<MenuItem>,

  // Mapping between the raw numeric ids and string ids
  raw_ids: HashMap<u32, String>,
}

impl Menu {
  pub fn from(mut items: Vec<MenuItem>) -> Result<Self> {
    // Generate the raw ids, also checking for duplicate ids
    let mut raw_ids: HashMap<u32, String> = HashMap::new();
    let mut ids: HashSet<String> = HashSet::new();
    let mut current = 0;
    for item in items.iter_mut() {
      Self::generate_raw_id(&mut raw_ids, &mut ids, &mut current, item)?;
    }

    Ok(Self { items, raw_ids })
  }

  pub fn to_json(&self) -> Result<String> {
    Ok(serde_json::to_string(&self.items)?)
  }

  pub fn get_item_id(&self, raw_id: u32) -> Option<String> {
    self.raw_ids.get(&raw_id).cloned()
  }

  fn generate_raw_id(raw_ids: &mut HashMap<u32, String>, ids: &mut HashSet<String>, current: &mut u32, item: &mut MenuItem) -> Result<()> {
    match item {
      MenuItem::Simple(simple_item) => {
        if ids.contains::<str>(&simple_item.id) {  // Duplicate id, throw error
          Err(MenuError::DuplicateMenuId(simple_item.id.to_string()).into())
        } else {
          ids.insert(simple_item.id.to_string());
          raw_ids.insert(*current, simple_item.id.to_string());
          simple_item.raw_id = Some(*current);
          *current += 1;
          Ok(())
        }
      }
      MenuItem::Sub(SubMenuItem { items, ..}) => {
        for sub_item in items.iter_mut() {
          Self::generate_raw_id(raw_ids, ids, current, sub_item)?
        }
        Ok(())
      }
      MenuItem::Separator => Ok(()),
    }
  }
}

#[derive(Error, Debug)]
pub enum MenuError {
  #[error("json serialization error")]
  Serialization(#[from] serde_json::Error),

  #[error("two or more menu items share the same id")]
  DuplicateMenuId(String),
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
  id: String,
  label: String,
  raw_id: Option<u32>,
}

impl SimpleMenuItem {
  pub fn new(id: &str, label: &str) -> Self {
    Self {
      id: id.to_string(),
      label: label.to_string(),
      raw_id: None,
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubMenuItem {
  label: String,
  items: Vec<MenuItem>,
}

impl SubMenuItem {
  pub fn new(label: &str, items: Vec<MenuItem>) -> Self {
    Self {
      label: label.to_string(),
      items,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_context_menu_serializes_correctly() {
    let menu = Menu::from(vec![
      MenuItem::Simple(SimpleMenuItem::new(
        "open",
        "Open",
      )),
      MenuItem::Separator,
      MenuItem::Sub(SubMenuItem::new(
        "Sub",
        vec![
          MenuItem::Simple(SimpleMenuItem::new(
            "sub1",
            "Sub 1",
          )),
          MenuItem::Simple(SimpleMenuItem::new(
            "sub2",
            "Sub 2",
          )),
        ]),
      ),
    ]).unwrap();
    assert_eq!(
      menu.to_json().unwrap(),
      r#"[{"type":"simple","id":"open","label":"Open","raw_id":0},{"type":"separator"},{"type":"sub","label":"Sub","items":[{"type":"simple","id":"sub1","label":"Sub 1","raw_id":1},{"type":"simple","id":"sub2","label":"Sub 2","raw_id":2}]}]"#
    );
  }

  #[test]
  fn test_context_menu_raw_ids_work_correctly() {
    let menu = Menu::from(vec![
      MenuItem::Simple(SimpleMenuItem::new(
        "open",
        "Open",
      )),
      MenuItem::Separator,
      MenuItem::Sub(SubMenuItem::new(
        "Sub",
        vec![
          MenuItem::Simple(SimpleMenuItem::new(
            "sub1",
            "Sub 1",
          )),
          MenuItem::Simple(SimpleMenuItem::new(
            "sub2",
            "Sub 2",
          )),
        ]),
      ),
    ]).unwrap();
    assert_eq!(
      menu.get_item_id(0).unwrap(),
      "open"
    );
    assert_eq!(
      menu.get_item_id(1).unwrap(),
      "sub1"
    );
    assert_eq!(
      menu.get_item_id(2).unwrap(),
      "sub2"
    );
  }

  #[test]
  fn test_context_menu_detects_duplicate_ids() {
    let menu = Menu::from(vec![
      MenuItem::Simple(SimpleMenuItem::new(
        "open",
        "Open",
      )),
      MenuItem::Separator,
      MenuItem::Sub(SubMenuItem::new(
        "Sub",
        vec![
          MenuItem::Simple(SimpleMenuItem::new(
            "open",
            "Sub 1",
          )),
          MenuItem::Simple(SimpleMenuItem::new(
            "sub2",
            "Sub 2",
          )),
        ]),
      ),
    ]);
    assert!(matches!(menu.unwrap_err().downcast::<MenuError>().unwrap(), 
            MenuError::DuplicateMenuId(id) if id == "open")); 
  }
}

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

use espanso_render::extension::choice::{ChoiceSelector, ChoiceSelectorResult};

use crate::gui::{SearchItem, SearchUI};

pub struct ChoiceSelectorAdapter<'a> {
  search_ui: &'a dyn SearchUI,
}

impl<'a> ChoiceSelectorAdapter<'a> {
  pub fn new(search_ui: &'a dyn SearchUI) -> Self {
    Self { search_ui }
  }
}

impl<'a> ChoiceSelector for ChoiceSelectorAdapter<'a> {
  fn show(&self, choices: &[espanso_render::extension::choice::Choice]) -> ChoiceSelectorResult {
    let items = convert_items(choices);
    match self.search_ui.show(&items, None) {
      Ok(Some(choice)) => ChoiceSelectorResult::Success(choice),
      Ok(None) => ChoiceSelectorResult::Aborted,
      Err(err) => ChoiceSelectorResult::Error(err),
    }
  }
}

fn convert_items(choices: &[espanso_render::extension::choice::Choice]) -> Vec<SearchItem> {
  choices
    .iter()
    .map(|choice| SearchItem {
      id: choice.id.to_string(),
      label: choice.label.to_string(),
      tag: None,
      is_builtin: false,
    })
    .collect()
}

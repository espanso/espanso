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

use unicase::UniCase;

use crate::event::Key;

use super::RollingItem;

#[derive(Debug)]
pub(crate) enum MatcherTreeRef<Id> {
  Matches(Vec<Id>),
  Node(Box<MatcherTreeNode<Id>>),
}

#[derive(Debug)]
pub(crate) struct MatcherTreeNode<Id> {
  pub word_separators: Option<MatcherTreeRef<Id>>,
  pub keys: Vec<(Key, MatcherTreeRef<Id>)>,
  pub chars: Vec<(String, MatcherTreeRef<Id>)>,
  pub chars_insensitive: Vec<(UniCase<String>, MatcherTreeRef<Id>)>,
}

impl <Id> Default for MatcherTreeNode<Id> {
  fn default() -> Self {
    Self {
      word_separators: None,
      keys: Vec::new(),
      chars: Vec::new(),
      chars_insensitive: Vec::new(),
    }
  }
}

impl <Id> MatcherTreeNode<Id> {
  // TODO: test
  pub fn from_items(items: &[RollingItem]) -> MatcherTreeNode<Id> {
    // TODO: implement the tree building algorithm
    todo!()
  }
}

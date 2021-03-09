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

use super::item::RollingItem;

#[derive(Debug)]
pub(crate) enum MatcherTreeRef {
  Matches(Vec<i32>),
  Node(Box<MatcherTreeNode>),
}

#[derive(Debug)]
pub(crate) struct MatcherTreeNode {
  pub word_separators: Option<MatcherTreeRef>,
  pub keys: Vec<(Key, MatcherTreeRef)>,
  pub chars: Vec<(String, MatcherTreeRef)>,
  pub chars_insensitive: Vec<(UniCase<String>, MatcherTreeRef)>,
}

impl MatcherTreeNode {
  // TODO: test
  pub fn from_items(items: &[RollingItem]) -> MatcherTreeNode {
    todo!()
  }
}


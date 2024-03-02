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

use super::{RollingItem, RollingMatch};

#[derive(Debug, PartialEq)]
pub(crate) enum MatcherTreeRef<Id> {
  Matches(Vec<Id>),
  Node(Box<MatcherTreeNode<Id>>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct MatcherTreeNode<Id> {
  pub word_separators: Option<MatcherTreeRef<Id>>,
  pub keys: Vec<(Key, MatcherTreeRef<Id>)>,
  pub chars: Vec<(String, MatcherTreeRef<Id>)>,
  pub chars_insensitive: Vec<(UniCase<String>, MatcherTreeRef<Id>)>,
}

impl<Id> Default for MatcherTreeNode<Id> {
  fn default() -> Self {
    Self {
      word_separators: None,
      keys: Vec::new(),
      chars: Vec::new(),
      chars_insensitive: Vec::new(),
    }
  }
}

impl<Id> MatcherTreeNode<Id>
where
  Id: Clone,
{
  pub fn from_matches(matches: &[RollingMatch<Id>]) -> MatcherTreeNode<Id> {
    let mut root = MatcherTreeNode::default();
    for m in matches {
      insert_items_recursively(m.id.clone(), &mut root, &m.items);
    }
    root
  }
}

fn insert_items_recursively<Id>(id: Id, node: &mut MatcherTreeNode<Id>, items: &[RollingItem]) {
  if items.is_empty() {
    return;
  }

  let item = items.first().unwrap();
  if items.len() == 1 {
    match item {
      RollingItem::WordSeparator => {
        let mut new_matches = Vec::new();
        if let Some(MatcherTreeRef::Matches(matches)) = node.word_separators.take() {
          new_matches.extend(matches);
        }
        new_matches.push(id);
        node.word_separators = Some(MatcherTreeRef::Matches(new_matches));
      }
      RollingItem::Key(key) => {
        if let Some(entry) = node.keys.iter_mut().find(|(k, _)| k == key) {
          if let MatcherTreeRef::Matches(matches) = &mut entry.1 {
            matches.push(id);
          } else {
            entry.1 = MatcherTreeRef::Matches(vec![id]);
          };
        } else {
          node
            .keys
            .push((key.clone(), MatcherTreeRef::Matches(vec![id])));
        }
      }
      RollingItem::Char(c) => {
        if let Some(entry) = node.chars.iter_mut().find(|(char, _)| char == c) {
          if let MatcherTreeRef::Matches(matches) = &mut entry.1 {
            matches.push(id);
          } else {
            entry.1 = MatcherTreeRef::Matches(vec![id]);
          };
        } else {
          node
            .chars
            .push((c.clone(), MatcherTreeRef::Matches(vec![id])));
        }
      }
      RollingItem::CharInsensitive(c) => {
        let uni_char = UniCase::new(c.clone());
        if let Some(entry) = node
          .chars_insensitive
          .iter_mut()
          .find(|(c, _)| c == &uni_char)
        {
          if let MatcherTreeRef::Matches(matches) = &mut entry.1 {
            matches.push(id);
          } else {
            entry.1 = MatcherTreeRef::Matches(vec![id]);
          };
        } else {
          node
            .chars_insensitive
            .push((uni_char, MatcherTreeRef::Matches(vec![id])));
        }
      }
    }
  } else {
    match item {
      RollingItem::WordSeparator => match node.word_separators.as_mut() {
        Some(MatcherTreeRef::Node(next_node)) => {
          insert_items_recursively(id, next_node.as_mut(), &items[1..]);
        }
        None => {
          let mut next_node = Box::<MatcherTreeNode<Id>>::default();
          insert_items_recursively(id, next_node.as_mut(), &items[1..]);
          node.word_separators = Some(MatcherTreeRef::Node(next_node));
        }
        _ => {}
      },
      RollingItem::Key(key) => {
        if let Some(entry) = node.keys.iter_mut().find(|(k, _)| k == key) {
          if let MatcherTreeRef::Node(next_node) = &mut entry.1 {
            insert_items_recursively(id, next_node, &items[1..]);
          }
        } else {
          let mut next_node = Box::<MatcherTreeNode<Id>>::default();
          insert_items_recursively(id, next_node.as_mut(), &items[1..]);
          node
            .keys
            .push((key.clone(), MatcherTreeRef::Node(next_node)));
        }
      }
      RollingItem::Char(c) => {
        if let Some(entry) = node.chars.iter_mut().find(|(char, _)| char == c) {
          if let MatcherTreeRef::Node(next_node) = &mut entry.1 {
            insert_items_recursively(id, next_node, &items[1..]);
          }
        } else {
          let mut next_node = Box::<MatcherTreeNode<Id>>::default();
          insert_items_recursively(id, next_node.as_mut(), &items[1..]);
          node
            .chars
            .push((c.clone(), MatcherTreeRef::Node(next_node)));
        }
      }
      RollingItem::CharInsensitive(c) => {
        let uni_char = UniCase::new(c.clone());
        if let Some(entry) = node
          .chars_insensitive
          .iter_mut()
          .find(|(c, _)| c == &uni_char)
        {
          if let MatcherTreeRef::Node(next_node) = &mut entry.1 {
            insert_items_recursively(id, next_node, &items[1..]);
          }
        } else {
          let mut next_node = Box::<MatcherTreeNode<Id>>::default();
          insert_items_recursively(id, next_node.as_mut(), &items[1..]);
          node
            .chars_insensitive
            .push((uni_char, MatcherTreeRef::Node(next_node)));
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::rolling::StringMatchOptions;

  #[test]
  fn generate_tree_from_items_simple_strings() {
    let root = MatcherTreeNode::from_matches(&[
      RollingMatch::from_string(1, "hi", &StringMatchOptions::default()),
      RollingMatch::from_string(2, "hey", &StringMatchOptions::default()),
      RollingMatch::from_string(3, "my", &StringMatchOptions::default()),
      RollingMatch::from_string(4, "myself", &StringMatchOptions::default()),
    ]);

    assert_eq!(
      root,
      MatcherTreeNode {
        chars: vec![
          (
            "h".to_string(),
            MatcherTreeRef::Node(Box::new(MatcherTreeNode {
              chars: vec![
                ("i".to_string(), MatcherTreeRef::Matches(vec![1])),
                (
                  "e".to_string(),
                  MatcherTreeRef::Node(Box::new(MatcherTreeNode {
                    chars: vec![("y".to_string(), MatcherTreeRef::Matches(vec![2]))],
                    ..Default::default()
                  }))
                ),
              ],
              ..Default::default()
            }))
          ),
          (
            "m".to_string(),
            MatcherTreeRef::Node(Box::new(MatcherTreeNode {
              chars: vec![("y".to_string(), MatcherTreeRef::Matches(vec![3])),],
              ..Default::default()
            }))
          )
        ],
        ..Default::default()
      }
    );
  }

  #[test]
  fn generate_tree_from_items_keys() {
    let root = MatcherTreeNode::from_matches(&[
      RollingMatch::from_items(
        1,
        &[
          RollingItem::Key(Key::ArrowUp),
          RollingItem::Key(Key::ArrowLeft),
        ],
      ),
      RollingMatch::from_items(
        2,
        &[
          RollingItem::Key(Key::ArrowUp),
          RollingItem::Key(Key::ArrowRight),
        ],
      ),
    ]);

    assert_eq!(
      root,
      MatcherTreeNode {
        keys: vec![(
          Key::ArrowUp,
          MatcherTreeRef::Node(Box::new(MatcherTreeNode {
            keys: vec![
              (Key::ArrowLeft, MatcherTreeRef::Matches(vec![1])),
              (Key::ArrowRight, MatcherTreeRef::Matches(vec![2])),
            ],
            ..Default::default()
          }))
        ),],
        ..Default::default()
      }
    );
  }

  #[test]
  fn generate_tree_from_items_mixed() {
    let root = MatcherTreeNode::from_matches(&[
      RollingMatch::from_items(
        1,
        &[
          RollingItem::Key(Key::ArrowUp),
          RollingItem::Key(Key::ArrowLeft),
        ],
      ),
      RollingMatch::from_string(
        2,
        "my",
        &StringMatchOptions {
          left_word: true,
          ..Default::default()
        },
      ),
      RollingMatch::from_string(
        3,
        "hi",
        &StringMatchOptions {
          left_word: true,
          right_word: true,
          ..Default::default()
        },
      ),
      RollingMatch::from_string(
        4,
        "no",
        &StringMatchOptions {
          case_insensitive: true,
          ..Default::default()
        },
      ),
    ]);

    assert_eq!(
      root,
      MatcherTreeNode {
        keys: vec![(
          Key::ArrowUp,
          MatcherTreeRef::Node(Box::new(MatcherTreeNode {
            keys: vec![(Key::ArrowLeft, MatcherTreeRef::Matches(vec![1])),],
            ..Default::default()
          }))
        ),],
        word_separators: Some(MatcherTreeRef::Node(Box::new(MatcherTreeNode {
          chars: vec![
            (
              "m".to_string(),
              MatcherTreeRef::Node(Box::new(MatcherTreeNode {
                chars: vec![("y".to_string(), MatcherTreeRef::Matches(vec![2])),],
                ..Default::default()
              }))
            ),
            (
              "h".to_string(),
              MatcherTreeRef::Node(Box::new(MatcherTreeNode {
                chars: vec![(
                  "i".to_string(),
                  MatcherTreeRef::Node(Box::new(MatcherTreeNode {
                    word_separators: Some(MatcherTreeRef::Matches(vec![3])),
                    ..Default::default()
                  }))
                ),],
                ..Default::default()
              }))
            ),
          ],
          ..Default::default()
        }))),
        chars_insensitive: vec![(
          UniCase::new("n".to_string()),
          MatcherTreeRef::Node(Box::new(MatcherTreeNode {
            chars_insensitive: vec![(
              UniCase::new("o".to_string()),
              MatcherTreeRef::Matches(vec![4])
            ),],
            ..Default::default()
          }))
        ),],
        ..Default::default()
      }
    );
  }
}

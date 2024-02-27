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

use super::split::*;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  // We need to match for both the new [[name]] syntax and the legacy {{name}} one
  static ref FIELD_REGEX: Regex = Regex::new(r"\{\{(.*?)\}\}|\[\[(.*?)\]\]").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
  Text(String),  // Text
  Field(String), // id: String
}

pub fn parse_layout(layout: &str) -> Vec<Vec<Token>> {
  let mut rows = Vec::new();

  for line in layout.lines() {
    let line = line.trim();

    // Skip empty lines
    if line.is_empty() {
      continue;
    }

    let mut row: Vec<Token> = Vec::new();

    let splitter = SplitCaptures::new(&FIELD_REGEX, line);

    // Get the individual tokens
    for state in splitter {
      match state {
        SplitState::Unmatched(text) => {
          if !text.is_empty() {
            row.push(Token::Text(text.to_owned()));
          }
        }
        SplitState::Captured(caps) => {
          if let Some(name) = caps.get(1) {
            let name = name.as_str().to_owned();
            row.push(Token::Field(name));
          } else if let Some(name) = caps.get(2) {
            let name = name.as_str().to_owned();
            row.push(Token::Field(name));
          }
        }
      }
    }

    rows.push(row);
  }

  rows
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_layout() {
    let layout = "Hey [[name]],\nHow are you?\n  \nCheers";
    let result = parse_layout(layout);
    assert_eq!(
      result,
      vec![
        vec![
          Token::Text("Hey ".to_owned()),
          Token::Field("name".to_owned()),
          Token::Text(",".to_owned())
        ],
        vec![Token::Text("How are you?".to_owned())],
        vec![Token::Text("Cheers".to_owned())],
      ]
    );
  }

  #[test]
  fn test_parse_layout_2() {
    let layout = "Hey [[name]] [[surname]],";
    let result = parse_layout(layout);
    assert_eq!(
      result,
      vec![vec![
        Token::Text("Hey ".to_owned()),
        Token::Field("name".to_owned()),
        Token::Text(" ".to_owned()),
        Token::Field("surname".to_owned()),
        Token::Text(",".to_owned())
      ],]
    );
  }

  #[test]
  fn test_parse_layout_legacy_syntax() {
    let layout = "Hey {{name}},\nHow are you?\n  \nCheers";
    let result = parse_layout(layout);
    assert_eq!(
      result,
      vec![
        vec![
          Token::Text("Hey ".to_owned()),
          Token::Field("name".to_owned()),
          Token::Text(",".to_owned())
        ],
        vec![Token::Text("How are you?".to_owned())],
        vec![Token::Text("Cheers".to_owned())],
      ]
    );
  }
}

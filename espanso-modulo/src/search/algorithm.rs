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

use std::collections::HashSet;

use crate::sys::search::types::SearchItem;

type FilterCallback = dyn Fn(&str, &[SearchItem]) -> Vec<usize>;

pub fn get_algorithm(name: &str, use_command_filter: bool) -> Box<FilterCallback> {
  let search_algorithm: Box<FilterCallback> = match name {
    "exact" => Box::new(exact_match),
    "iexact" => Box::new(case_insensitive_exact_match),
    "ikey" => Box::new(case_insensitive_keyword),
    _ => panic!("unknown search algorithm: {name}"),
  };

  if use_command_filter {
    command_filter(search_algorithm)
  } else {
    search_algorithm
  }
}

fn exact_match(query: &str, items: &[SearchItem]) -> Vec<usize> {
  items
    .iter()
    .enumerate()
    .filter(|(_, item)| {
      item.label.contains(query)
        || item.trigger.as_deref().map_or(false, |t| t.contains(query))
        || item.search_terms.iter().any(|term| term.contains(query))
    })
    .map(|(i, _)| i)
    .collect()
}

fn case_insensitive_exact_match(query: &str, items: &[SearchItem]) -> Vec<usize> {
  let lowercase_query = query.to_lowercase();
  items
    .iter()
    .enumerate()
    .filter(|(_, item)| {
      item.label.to_lowercase().contains(&lowercase_query)
        || item
          .trigger
          .as_deref()
          .map_or(false, |t| t.to_lowercase().contains(query))
        || item
          .search_terms
          .iter()
          .any(|term| term.to_lowercase().contains(&lowercase_query))
    })
    .map(|(i, _)| i)
    .collect()
}

fn case_insensitive_keyword(query: &str, items: &[SearchItem]) -> Vec<usize> {
  let lowercase_query = query.to_lowercase();
  let keywords: Vec<&str> = lowercase_query.split_whitespace().collect();
  items
    .iter()
    .enumerate()
    .filter(|(_, item)| {
      for keyword in &keywords {
        if !item.label.to_lowercase().contains(keyword)
          && !item
            .trigger
            .as_deref()
            .map_or(false, |t| t.to_lowercase().contains(keyword))
          && !item
            .search_terms
            .iter()
            .any(|term| term.to_lowercase().contains(keyword))
        {
          return false;
        }
      }

      true
    })
    .map(|(i, _)| i)
    .collect()
}

fn command_filter(search_algorithm: Box<FilterCallback>) -> Box<FilterCallback> {
  Box::new(move |query, items| {
    let (valid_ids, trimmed_query) = if query.starts_with('>') {
      (
        items
          .iter()
          .enumerate()
          .filter(|(_, item)| item.is_builtin)
          .map(|(i, _)| i)
          .collect::<HashSet<usize>>(),
        query.trim_start_matches('>'),
      )
    } else {
      (
        items
          .iter()
          .enumerate()
          .filter(|(_, item)| !item.is_builtin)
          .map(|(i, _)| i)
          .collect::<HashSet<usize>>(),
        query,
      )
    };

    let results = search_algorithm(trimmed_query, items);

    results
      .into_iter()
      .filter(|id| valid_ids.contains(id))
      .collect()
  })
}

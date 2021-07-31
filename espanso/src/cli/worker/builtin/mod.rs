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

use std::cell::Cell;

use crate::engine::event::EventType;

use super::context::Context;

mod debug;

const MIN_BUILTIN_MATCH_ID: i32 = 1_000_000_000;

pub struct BuiltInMatch {
  pub id: i32,
  pub label: &'static str,
  pub triggers: Vec<String>,
  pub action: fn(context: &dyn Context) -> EventType,
}

pub fn get_builtin_matches() -> Vec<BuiltInMatch> {
  vec![
    debug::create_match_show_active_config_info(),
  ]
}

pub fn is_builtin_match(id: i32) -> bool {
  id >= MIN_BUILTIN_MATCH_ID
}

thread_local! {
  static CURRENT_BUILTIN_MATCH_ID: Cell<i32> = Cell::new(MIN_BUILTIN_MATCH_ID);
}

fn generate_next_builtin_id() -> i32 {
  CURRENT_BUILTIN_MATCH_ID.with(|value| {
    let current = value.get();
    value.set(current + 1);
    current
  })
}

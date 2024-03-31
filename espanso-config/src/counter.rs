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

use std::sync::atomic::{AtomicI32, Ordering};

thread_local! {
  // TODO: if thread local, we probably don't need an atomic
  static STRUCT_COUNTER: AtomicI32 = const { AtomicI32::new(0) };
}

pub type StructId = i32;

/// Some structs need a unique id.
/// In order to generate it, we use an atomic static variable
/// that is incremented for each struct.
pub fn next_id() -> StructId {
  STRUCT_COUNTER.with(|count| count.fetch_add(1, Ordering::SeqCst))
}

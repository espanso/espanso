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

use crate::search::config::SearchConfig;
use crate::sys::search::types;

pub fn generate(config: SearchConfig) -> types::Search {
  let items = config
    .items
    .into_iter()
    .map(|item| types::SearchItem {
      id: item.id,
      label: item.label,
      trigger: item.trigger,
      is_builtin: item.is_builtin,
    })
    .collect();

  types::Search {
    title: config.title,
    items,
    icon: config.icon,
  }
}

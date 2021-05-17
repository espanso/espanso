/*
 * This file is part of espanso.
 *
 * Copyright  id: (), label: () id: (), label: () id: (), label: ()(C) 2019-2021 Federico Terzi
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

use super::super::Middleware;
use crate::engine::{event::{Event, EventType, ui::{MenuItem, ShowContextMenuEvent, SimpleMenuItem}}};

pub struct ContextMenuMiddleware {
}

impl ContextMenuMiddleware {
  pub fn new() -> Self {
    Self {}
  }
}

impl Middleware for ContextMenuMiddleware {
  fn name(&self) -> &'static str {
    "context_menu"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    if let EventType::TrayIconClicked = event.etype {
      // TODO: fetch top matches for the active config to be added

      // TODO: my idea is to use a set of reserved u32 ids for built-in
      // actions such as Exit, Open Editor etc
      // then we need some u32 for the matches, so we need to create
      // a mapping structure match_id <-> context-menu-id
      return Event::caused_by(
        event.source_id,
        EventType::ShowContextMenu(ShowContextMenuEvent {
          // TODO: add actual entries
          items: vec![
            MenuItem::Simple(SimpleMenuItem {
              id: 0,
              label: "Exit espanso".to_string(),
            })
          ]
        }),
      )
    }

    // TODO: handle context menu clicks

    event
  }
}

// TODO: test

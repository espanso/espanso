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

use std::{
  cell::RefCell,
};

use super::super::Middleware;
use crate::engine::event::{Event, EventType, ui::{IconStatus, IconStatusChangeEvent}};

pub struct IconStatusMiddleware {
  enabled: RefCell<bool>,
  secure_input_enabled: RefCell<bool>,
}

impl IconStatusMiddleware {
  pub fn new() -> Self {
    Self {
      enabled: RefCell::new(true),
      secure_input_enabled: RefCell::new(false),
    }
  }
}

impl Middleware for IconStatusMiddleware {
  fn name(&self) -> &'static str {
    "icon_status"
  }

  fn next(&self, event: Event, dispatch: &mut dyn FnMut(Event)) -> Event {
    let mut enabled = self.enabled.borrow_mut();
    let mut secure_input_enabled = self.secure_input_enabled.borrow_mut();

    let mut did_update = true;
    match &event.etype {
      EventType::Enabled => *enabled = true,
      EventType::Disabled => *enabled = false,
      EventType::SecureInputEnabled => *secure_input_enabled = true,
      EventType::SecureInputDisabled => *secure_input_enabled = false,
      _ => did_update = false,
    }

    if did_update {
      let status = if *enabled {
        if *secure_input_enabled {
          IconStatus::SecureInputDisabled
        } else {
          IconStatus::Enabled
        }
      } else {
        IconStatus::Disabled
      };

      dispatch(Event::caused_by(event.source_id, EventType::IconStatusChange(IconStatusChangeEvent {
        status,
      })));
    }

    event
  }
}

// TODO: test

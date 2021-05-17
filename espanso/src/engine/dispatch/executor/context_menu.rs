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

use super::super::{Event, Executor};
use crate::engine::event::{ui::MenuItem, EventType};
use anyhow::Result;
use log::error;

pub trait ContextMenuHandler {
  fn show_context_menu(&self, items: &[MenuItem]) -> Result<()>;
}

pub struct ContextMenuExecutor<'a> {
  handler: &'a dyn ContextMenuHandler,
}

impl<'a> ContextMenuExecutor<'a> {
  pub fn new(handler: &'a dyn ContextMenuHandler) -> Self {
    Self { handler }
  }
}

impl<'a> Executor for ContextMenuExecutor<'a> {
  fn execute(&self, event: &Event) -> bool {
    if let EventType::ShowContextMenu(context_menu_event) = &event.etype {
      if let Err(error) = self.handler.show_context_menu(&context_menu_event.items) {
        error!("context menu handler reported an error: {:?}", error);
      }

      return true;
    }

    false
  }
}

// TODO: test

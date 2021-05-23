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
use crate::engine::event::{EventType, ui::{IconStatus}};
use anyhow::Result;
use log::error;

pub trait IconHandler {
  fn update_icon(&self, status: &IconStatus) -> Result<()>;
}

pub struct IconUpdateExecutor<'a> {
  handler: &'a dyn IconHandler,
}

impl<'a> IconUpdateExecutor<'a> {
  pub fn new(handler: &'a dyn IconHandler) -> Self {
    Self { handler }
  }
}

impl<'a> Executor for IconUpdateExecutor<'a> {
  fn execute(&self, event: &Event) -> bool {
    if let EventType::IconStatusChange(m_event) = &event.etype {
      if let Err(error) = self.handler.update_icon(&m_event.status) {
        error!("icon handler reported an error: {:?}", error);
      }

      return true;
    }

    false
  }
}

// TODO: test

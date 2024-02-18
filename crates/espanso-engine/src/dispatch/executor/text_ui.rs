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

use crate::event::EventType;
use crate::{dispatch::Executor, event::Event};
use anyhow::Result;
use log::error;

pub trait TextUIHandler {
  fn show_text(&self, title: &str, text: &str) -> Result<()>;
  fn show_logs(&self) -> Result<()>;
}

pub struct TextUIExecutor<'a> {
  handler: &'a dyn TextUIHandler,
}

impl<'a> TextUIExecutor<'a> {
  pub fn new(handler: &'a dyn TextUIHandler) -> Self {
    Self { handler }
  }
}

impl<'a> Executor for TextUIExecutor<'a> {
  fn execute(&self, event: &Event) -> bool {
    if let EventType::ShowText(show_text_event) = &event.etype {
      if let Err(error) = self
        .handler
        .show_text(&show_text_event.title, &show_text_event.text)
      {
        error!("text UI handler reported an error: {:?}", error);
      }

      return true;
    } else if let EventType::ShowLogs = &event.etype {
      if let Err(error) = self.handler.show_logs() {
        error!("text UI handler reported an error: {:?}", error);
      }

      return true;
    }

    false
  }
}

// TODO: test

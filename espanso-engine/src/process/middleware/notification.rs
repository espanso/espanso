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

use super::super::Middleware;
use crate::event::{Event, EventType};

pub trait NotificationManager {
  fn notify_status_change(&self, enabled: bool);
  fn notify_rendering_error(&self);
}

pub struct NotificationMiddleware<'a> {
  notification_manager: &'a dyn NotificationManager,
}

impl<'a> NotificationMiddleware<'a> {
  pub fn new(notification_manager: &'a dyn NotificationManager) -> Self {
    Self {
      notification_manager,
    }
  }
}

impl<'a> Middleware for NotificationMiddleware<'a> {
  fn name(&self) -> &'static str {
    "notification"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    match &event.etype {
      EventType::Enabled => self.notification_manager.notify_status_change(true),
      EventType::Disabled => self.notification_manager.notify_status_change(false),
      EventType::RenderingError => self.notification_manager.notify_rendering_error(),
      _ => {}
    }

    event
  }
}

// TODO: test

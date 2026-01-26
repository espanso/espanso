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

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use super::super::Middleware;
use crate::event::{Event, EventType};

pub struct EnabledStateMiddleware {
    enabled_state: Arc<AtomicBool>,
}

impl EnabledStateMiddleware {
    pub fn new(enabled_state: Arc<AtomicBool>) -> Self {
        Self { enabled_state }
    }
}

impl Middleware for EnabledStateMiddleware {
    fn name(&self) -> &'static str {
        "enabled_state"
    }

    fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
        match event.etype {
            EventType::Enabled => self.enabled_state.store(true, Ordering::SeqCst),
            EventType::Disabled => self.enabled_state.store(false, Ordering::SeqCst),
            _ => {}
        }

        event
    }
}

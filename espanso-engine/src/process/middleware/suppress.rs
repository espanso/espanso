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

use log::trace;

use super::super::Middleware;
use crate::event::{Event, EventType};

pub trait EnabledStatusProvider {
    fn is_config_enabled(&self) -> bool;
}

pub struct SuppressMiddleware<'a> {
    provider: &'a dyn EnabledStatusProvider,
}

impl<'a> SuppressMiddleware<'a> {
    pub fn new(provider: &'a dyn EnabledStatusProvider) -> Self {
        Self { provider }
    }
}

impl<'a> Middleware for SuppressMiddleware<'a> {
    fn name(&self) -> &'static str {
        "suppress"
    }

    fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
        if let EventType::MatchesDetected(_) = event.etype {
            if !self.provider.is_config_enabled() {
                trace!("suppressing match detected event as active config has enable=false");
                return Event::caused_by(event.source_id, EventType::NOOP);
            }
        }

        event
    }
}

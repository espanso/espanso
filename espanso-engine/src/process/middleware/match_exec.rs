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

use log::warn;

use super::super::Middleware;
use crate::event::{
    internal::{DetectedMatch, MatchesDetectedEvent},
    Event, EventType,
};

pub trait MatchResolver {
    fn find_matches_from_trigger(&self, trigger: &str) -> Vec<DetectedMatch>;
}

pub struct MatchExecRequestMiddleware<'a> {
    match_resolver: &'a dyn MatchResolver,
}

impl<'a> MatchExecRequestMiddleware<'a> {
    pub fn new(match_resolver: &'a dyn MatchResolver) -> Self {
        Self { match_resolver }
    }
}

impl<'a> Middleware for MatchExecRequestMiddleware<'a> {
    fn name(&self) -> &'static str {
        "match_exec_request"
    }

    fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
        if let EventType::MatchExecRequest(m_event) = &event.etype {
            let mut matches = if let Some(trigger) = &m_event.trigger {
                self.match_resolver.find_matches_from_trigger(trigger)
            } else {
                Vec::new()
            };

            // Inject the request args into the detected matches
            for m in &mut matches {
                for (key, value) in &m_event.args {
                    m.args.insert(key.to_string(), value.to_string());
                }
            }

            if matches.is_empty() {
                warn!("received match exec request, but no matches have been found for the given query.");
                return Event::caused_by(event.source_id, EventType::NOOP);
            }

            return Event::caused_by(
                event.source_id,
                EventType::MatchesDetected(MatchesDetectedEvent {
                    matches,
                    is_search: false,
                }),
            );
        }

        event
    }
}

// TODO: test

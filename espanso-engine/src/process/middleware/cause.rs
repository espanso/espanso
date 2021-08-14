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
use crate::{event::{Event, EventType, effect::TriggerCompensationEvent, internal::CauseCompensatedMatchEvent}};

pub struct CauseCompensateMiddleware {}

impl CauseCompensateMiddleware {
  pub fn new() -> Self {
    Self {}
  }
}

impl Middleware for CauseCompensateMiddleware {
  fn name(&self) -> &'static str {
    "cause_compensate"
  }

  fn next(&self, event: Event, dispatch: &mut dyn FnMut(Event)) -> Event {
    if let EventType::MatchSelected(m_event) = &event.etype {
      let compensated_event =
        Event::caused_by(event.source_id, EventType::CauseCompensatedMatch(CauseCompensatedMatchEvent { m: m_event.chosen.clone() }));

      if let Some(trigger) = &m_event.chosen.trigger {
        dispatch(compensated_event);

        // Before the event, place a trigger compensation
        return Event::caused_by(event.source_id, EventType::TriggerCompensation(TriggerCompensationEvent {
          trigger: trigger.clone(),
          left_separator: m_event.chosen.left_separator.clone(),
        }));
      } else {
        return compensated_event;
      }
    }

    event
  }
}

// TODO: test

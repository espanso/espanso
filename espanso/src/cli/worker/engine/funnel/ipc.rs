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

use crossbeam::channel::{Receiver, Select, SelectedOperation};

use espanso_engine::{
  event::{Event, EventType},
  funnel,
};
use log::warn;

use super::sequencer::Sequencer;

pub struct IpcEventSource<'a> {
  pub ipc_event_receiver: Receiver<EventType>,
  pub sequencer: &'a Sequencer,
}

impl<'a> IpcEventSource<'a> {
  pub fn new(ipc_event_receiver: Receiver<EventType>, sequencer: &'a Sequencer) -> Self {
    IpcEventSource {
      ipc_event_receiver,
      sequencer,
    }
  }
}

impl<'a> funnel::Source<'a> for IpcEventSource<'a> {
  fn register(&'a self, select: &mut Select<'a>) -> usize {
    select.recv(&self.ipc_event_receiver)
  }

  fn receive(&self, op: SelectedOperation) -> Option<Event> {
    let ipc_event = op
      .recv(&self.ipc_event_receiver)
      .expect("unable to select data from IpcEventSource receiver");

    // Execute only events that have been whitelisted
    if !is_event_type_allowed(&ipc_event) {
      warn!(
        "received black-listed event from IPC stream, blocking it: {:?}",
        ipc_event
      );
      return None;
    }

    Some(Event {
      source_id: self.sequencer.next_id(),
      etype: ipc_event,
    })
  }
}

fn is_event_type_allowed(event: &EventType) -> bool {
  matches!(
    event,
    EventType::MatchExecRequest(_)
      | EventType::ShowSearchBar
      | EventType::ShowConfigFolder
      | EventType::DisableRequest
      | EventType::EnableRequest
      | EventType::ToggleRequest
  )
}

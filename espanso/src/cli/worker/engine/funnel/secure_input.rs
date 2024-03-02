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

use crate::cli::worker::secure_input::SecureInputEvent;
use espanso_engine::{
    event::{internal::SecureInputEnabledEvent, Event, EventType},
    funnel,
};

use super::sequencer::Sequencer;

pub struct SecureInputSource<'a> {
    pub receiver: Receiver<SecureInputEvent>,
    pub sequencer: &'a Sequencer,
}

impl<'a> SecureInputSource<'a> {
    pub fn new(receiver: Receiver<SecureInputEvent>, sequencer: &'a Sequencer) -> Self {
        SecureInputSource {
            receiver,
            sequencer,
        }
    }
}

impl<'a> funnel::Source<'a> for SecureInputSource<'a> {
    fn register(&'a self, select: &mut Select<'a>) -> usize {
        select.recv(&self.receiver)
    }

    fn receive(&self, op: SelectedOperation) -> Option<Event> {
        let si_event = op
            .recv(&self.receiver)
            .expect("unable to select data from SecureInputSource receiver");

        Some(Event {
            source_id: self.sequencer.next_id(),
            etype: match si_event {
                SecureInputEvent::Disabled => EventType::SecureInputDisabled,
                SecureInputEvent::Enabled { app_name, app_path } => {
                    EventType::SecureInputEnabled(SecureInputEnabledEvent { app_name, app_path })
                }
            },
        })
    }
}

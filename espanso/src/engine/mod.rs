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

use log::{debug};

use self::{dispatch::Dispatcher, event::{Event, EventType}, funnel::{Funnel, FunnelResult}, process::Processor};

pub mod dispatch;
pub mod event;
pub mod process;
pub mod funnel;

pub struct Engine<'a> {
  funnel: &'a dyn Funnel,
  processor: &'a mut dyn Processor,
  dispatcher: &'a dyn Dispatcher,
}

impl <'a> Engine<'a> {
  pub fn new(funnel: &'a dyn Funnel, processor: &'a mut dyn Processor, dispatcher: &'a dyn Dispatcher) -> Self {
    Self {
      funnel,
      processor,
      dispatcher,
    }
  }

  pub fn run(&mut self) {
    'main: loop {
      match self.funnel.receive() {
        FunnelResult::Event(event) => {
          let processed_events = self.processor.process(event);
          for event in processed_events {
            if let EventType::Exit = &event.etype {
              debug!("exit event received, exiting engine");
              break 'main;
            }

            self.dispatcher.dispatch(event);
          }
        } 
        FunnelResult::EndOfStream => {
          debug!("end of stream received");
          break;
        }
      }
    }
  }
}

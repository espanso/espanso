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

use log::{debug, trace};
use std::collections::VecDeque;

use self::{
  dispatch::Dispatcher,
  event::Event,
  process::Processor,
  funnel::{Funnel, FunnelResult},
};

pub mod dispatch;
pub mod event;
pub mod process;
pub mod funnel;

pub struct Engine<TFunnel: Funnel, TProcessor: Processor, TDispatcher: Dispatcher> {
  funnel: TFunnel,
  processor: TProcessor,
  dispatcher: TDispatcher,
}

impl <TFunnel: Funnel, TProcessor: Processor, TDispatcher: Dispatcher> Engine<TFunnel, TProcessor, TDispatcher> {
  pub fn new(funnel: TFunnel, processor: TProcessor, dispatcher: TDispatcher) -> Self {
    Self {
      funnel,
      processor,
      dispatcher,
    }
  }

  pub fn run(&mut self) {
    loop {
      match self.funnel.receive() {
        FunnelResult::Event(event) => {
          trace!("received event from stream: {:?}", event);
          
          let processed_events = self.processor.process(event);
          for event in processed_events {
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

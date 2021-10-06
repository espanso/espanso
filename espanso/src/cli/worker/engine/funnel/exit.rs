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
  event::{Event, EventType, ExitMode},
  funnel,
};

use super::sequencer::Sequencer;

pub struct ExitSource<'a> {
  pub exit_signal: Receiver<ExitMode>,
  pub sequencer: &'a Sequencer,
}

impl<'a> ExitSource<'a> {
  pub fn new(exit_signal: Receiver<ExitMode>, sequencer: &'a Sequencer) -> Self {
    ExitSource {
      exit_signal,
      sequencer,
    }
  }
}

impl<'a> funnel::Source<'a> for ExitSource<'a> {
  fn register(&'a self, select: &mut Select<'a>) -> usize {
    select.recv(&self.exit_signal)
  }

  fn receive(&self, op: SelectedOperation) -> Event {
    let mode = op
      .recv(&self.exit_signal)
      .expect("unable to select data from ExitSource receiver");
    Event {
      source_id: self.sequencer.next_id(),
      etype: EventType::ExitRequested(mode),
    }
  }
}

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
  atomic::{AtomicU32, Ordering},
  Arc,
};

use espanso_engine::{event::SourceId, process::EventSequenceProvider};

#[derive(Clone)]
pub struct Sequencer {
  current_id: Arc<AtomicU32>,
}

impl Sequencer {
  pub fn new() -> Self {
    Self {
      current_id: Arc::new(AtomicU32::new(0)),
    }
  }

  pub fn next_id(&self) -> SourceId {
    self.current_id.fetch_add(1, Ordering::SeqCst)
  }
}

impl EventSequenceProvider for Sequencer {
  fn get_next_id(&self) -> SourceId {
    self.next_id()
  }
}

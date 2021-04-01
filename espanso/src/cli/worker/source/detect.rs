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
use espanso_detect::event::InputEvent;

use crate::engine::{event::Event, funnel::Source};

pub struct DetectorSource {
  receiver: Receiver<InputEvent>,
}

impl Source for DetectorSource {
  fn register(&self, select: &Select) -> i32 {
    todo!()
  }

  fn receive(&self, select: &SelectedOperation) -> Event {
    todo!()
  }
}

pub fn new() {
  todo!()
}
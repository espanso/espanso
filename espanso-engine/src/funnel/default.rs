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

use crossbeam::channel::Select;

use super::{Funnel, FunnelResult, Source};

pub struct DefaultFunnel<'a> {
  sources: &'a [&'a dyn Source<'a>],
}

impl<'a> DefaultFunnel<'a> {
  pub fn new(sources: &'a [&'a dyn Source<'a>]) -> Self {
    Self { sources }
  }
}

impl<'a> Funnel for DefaultFunnel<'a> {
  fn receive(&self) -> FunnelResult {
    let mut select = Select::new();

    // First register all the sources to the select operation
    for source in self.sources.iter() {
      source.register(&mut select);
    }

    // Wait for the first source (blocking operation)
    let op = select.select();
    let source = self
      .sources
      .get(op.index())
      .expect("invalid source index returned by select operation");

    // Receive (and convert) the event
    let event = source.receive(op);
    FunnelResult::Event(event)
  }
}

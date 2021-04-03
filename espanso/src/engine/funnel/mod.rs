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

use crossbeam::{channel::Select, channel::SelectedOperation};

use self::default::DefaultFunnel;

use super::Event;

mod default;

pub trait Source<'a> {
  fn register<'b>(&'a self, select: &mut Select<'a>) -> usize;
  fn receive<'b>(&'a self, op: SelectedOperation) -> Event;
}

pub trait Funnel {
  fn receive(&self) -> FunnelResult;
}

pub enum FunnelResult {
  Event(Event),
  EndOfStream,
}

pub fn default<'a>(sources: &'a [&'a dyn Source<'a>]) -> impl Funnel + 'a {
  DefaultFunnel::new(sources)
}
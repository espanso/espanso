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
use espanso_ui::event::UIEvent;

use crate::engine::{
  event::{input::ContextMenuClickedEvent, Event, EventType},
  funnel,
};

use super::sequencer::Sequencer;

pub struct UISource<'a> {
  pub ui_receiver: Receiver<UIEvent>,
  pub sequencer: &'a Sequencer,
}

impl<'a> UISource<'a> {
  pub fn new(ui_receiver: Receiver<UIEvent>, sequencer: &'a Sequencer) -> Self {
    UISource {
      ui_receiver,
      sequencer,
    }
  }
}

impl<'a> funnel::Source<'a> for UISource<'a> {
  fn register(&'a self, select: &mut Select<'a>) -> usize {
    select.recv(&self.ui_receiver)
  }

  fn receive(&self, op: SelectedOperation) -> Event {
    let ui_event = op
      .recv(&self.ui_receiver)
      .expect("unable to select data from UISource receiver");

    Event {
      source_id: self.sequencer.next_id(),
      etype: match ui_event {
        UIEvent::TrayIconClick => EventType::TrayIconClicked,
        UIEvent::ContextMenuClick(context_item_id) => {
          EventType::ContextMenuClicked(ContextMenuClickedEvent { context_item_id })
        }
        UIEvent::Heartbeat => EventType::Heartbeat,
      },
    }
  }
}

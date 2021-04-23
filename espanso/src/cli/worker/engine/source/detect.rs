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

use anyhow::Result;
use crossbeam::channel::{Receiver, Select, SelectedOperation};
use espanso_detect::{event::InputEvent, Source};
use log::{error, trace};

use crate::engine::{
  event::{
    input::{Key, KeyboardEvent, Status, Variant},
    Event,
  },
  funnel, process,
};
use thiserror::Error;

pub struct DetectSource {
  receiver: Receiver<InputEvent>,
}

impl<'a> funnel::Source<'a> for DetectSource {
  fn register(&'a self, select: &mut Select<'a>) -> usize {
    select.recv(&self.receiver)
  }

  fn receive(&self, op: SelectedOperation) -> Event {
    let input_event = op
      .recv(&self.receiver)
      .expect("unable to select data from DetectSource receiver");
    match input_event {
      InputEvent::Keyboard(keyboard_event) => Event::Keyboard(KeyboardEvent {
        key: keyboard_event.key.into(),
        value: keyboard_event.value,
        status: keyboard_event.status.into(),
        variant: keyboard_event.variant.map(|variant| variant.into()),
      }),
      InputEvent::Mouse(mouse_event) => todo!(),  // TODO
      InputEvent::HotKey(_) => todo!(), // TODO
    }
  }
}

// TODO: pass options
pub fn init_and_spawn() -> Result<DetectSource> {
  let (sender, receiver) = crossbeam::channel::unbounded();
  let (init_tx, init_rx) = crossbeam::channel::unbounded();

  if let Err(error) = std::thread::Builder::new()
    .name("detect thread".to_string())
    .spawn(
      move || match espanso_detect::get_source(Default::default()) {
        Ok(mut source) => {
          if source.initialize().is_err() {
            init_tx
              .send(false)
              .expect("unable to send to the init_tx channel");
          } else {
            init_tx
              .send(true)
              .expect("unable to send to the init_tx channel");

            source.eventloop(Box::new(move |event| {
              sender
                .send(event)
                .expect("unable to send to the source channel");
            })).expect("detect eventloop crashed");
          }
        }
        Err(error) => {
          error!("cannot initialize event source: {:?}", error);
          init_tx
            .send(false)
            .expect("unable to send to the init_tx channel");
        }
      },
    )
  {
    error!("detection thread initialization failed: {:?}", error);
    return Err(DetectSourceError::ThreadInitFailed.into());
  }

  // Wait for the initialization status
  let has_initialized = init_rx
    .recv()
    .expect("unable to receive from the init_rx channel");
  if !has_initialized {
    return Err(DetectSourceError::InitFailed.into());
  }

  Ok(DetectSource { receiver })
}

#[derive(Error, Debug)]
pub enum DetectSourceError {
  #[error("detection thread initialization failed")]
  ThreadInitFailed,

  #[error("detection source initialization failed")]
  InitFailed,
}

impl From<espanso_detect::event::Key> for Key {
  fn from(key: espanso_detect::event::Key) -> Self {
    match key {
      espanso_detect::event::Key::Alt => Key::Alt,
      espanso_detect::event::Key::CapsLock => Key::CapsLock,
      espanso_detect::event::Key::Control => Key::Control,
      espanso_detect::event::Key::Meta => Key::Meta,
      espanso_detect::event::Key::NumLock => Key::NumLock,
      espanso_detect::event::Key::Shift => Key::Shift,
      espanso_detect::event::Key::Enter => Key::Enter,
      espanso_detect::event::Key::Tab => Key::Tab,
      espanso_detect::event::Key::Space => Key::Space,
      espanso_detect::event::Key::ArrowDown => Key::ArrowDown,
      espanso_detect::event::Key::ArrowLeft => Key::ArrowLeft,
      espanso_detect::event::Key::ArrowRight => Key::ArrowRight,
      espanso_detect::event::Key::ArrowUp => Key::ArrowUp,
      espanso_detect::event::Key::End => Key::End,
      espanso_detect::event::Key::Home => Key::Home,
      espanso_detect::event::Key::PageDown => Key::PageDown,
      espanso_detect::event::Key::PageUp => Key::PageUp,
      espanso_detect::event::Key::Escape => Key::Escape,
      espanso_detect::event::Key::Backspace => Key::Backspace,
      espanso_detect::event::Key::F1 => Key::F1,
      espanso_detect::event::Key::F2 => Key::F2,
      espanso_detect::event::Key::F3 => Key::F3,
      espanso_detect::event::Key::F4 => Key::F4,
      espanso_detect::event::Key::F5 => Key::F5,
      espanso_detect::event::Key::F6 => Key::F6,
      espanso_detect::event::Key::F7 => Key::F7,
      espanso_detect::event::Key::F8 => Key::F8,
      espanso_detect::event::Key::F9 => Key::F9,
      espanso_detect::event::Key::F10 => Key::F10,
      espanso_detect::event::Key::F11 => Key::F11,
      espanso_detect::event::Key::F12 => Key::F12,
      espanso_detect::event::Key::F13 => Key::F13,
      espanso_detect::event::Key::F14 => Key::F14,
      espanso_detect::event::Key::F15 => Key::F15,
      espanso_detect::event::Key::F16 => Key::F16,
      espanso_detect::event::Key::F17 => Key::F17,
      espanso_detect::event::Key::F18 => Key::F18,
      espanso_detect::event::Key::F19 => Key::F19,
      espanso_detect::event::Key::F20 => Key::F20,
      espanso_detect::event::Key::Other(code) => Key::Other(code),
    }
  }
}

impl From<espanso_detect::event::Variant> for Variant {
  fn from(variant: espanso_detect::event::Variant) -> Self {
    match variant {
      espanso_detect::event::Variant::Left => Variant::Left,
      espanso_detect::event::Variant::Right => Variant::Right,
    }
  }
}

impl From<espanso_detect::event::Status> for Status {
  fn from(status: espanso_detect::event::Status) -> Self {
    match status {
      espanso_detect::event::Status::Pressed => Status::Pressed,
      espanso_detect::event::Status::Released => Status::Released,
    }
  }
}

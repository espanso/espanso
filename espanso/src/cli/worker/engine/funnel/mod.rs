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
use espanso_detect::event::{InputEvent, KeyboardEvent, Status};
use log::{error};
use thiserror::Error;

use detect::DetectSource;

use self::{modifier::{Modifier, ModifierStateStore}, sequencer::Sequencer};

pub mod detect;
pub mod exit;
pub mod modifier;
pub mod sequencer;
pub mod ui;

// TODO: pass options
pub fn init_and_spawn() -> Result<(DetectSource, ModifierStateStore, Sequencer)> {
  let (sender, receiver) = crossbeam::channel::unbounded();
  let (init_tx, init_rx) = crossbeam::channel::unbounded();

  let modifier_state_store = ModifierStateStore::new();
  let sequencer = Sequencer::new();

  let state_store_clone = modifier_state_store.clone();
  let sequencer_clone = sequencer.clone();
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

            source
              .eventloop(Box::new(move |event| {
                // Update the modifiers state
                if let Some((modifier, is_pressed)) = get_modifier_status(&event) {
                  state_store_clone.update_state(modifier, is_pressed);
                }

                // Generate a monotonically increasing id for the current event
                let source_id = sequencer_clone.next_id();

                sender
                  .send((event, source_id))
                  .expect("unable to send to the source channel");
              }))
              .expect("detect eventloop crashed");
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

  Ok((DetectSource { receiver }, modifier_state_store, sequencer))
}

#[derive(Error, Debug)]
pub enum DetectSourceError {
  #[error("detection thread initialization failed")]
  ThreadInitFailed,

  #[error("detection source initialization failed")]
  InitFailed,
}

fn get_modifier_status(event: &InputEvent) -> Option<(Modifier, bool)> {
  match event {
    InputEvent::Keyboard(KeyboardEvent {
      key,
      status,
      value: _,
      variant: _,
      code: _,
    }) => {
      let is_pressed = *status == Status::Pressed;
      match key {
        espanso_detect::event::Key::Alt => Some((Modifier::Alt, is_pressed)),
        espanso_detect::event::Key::Control => Some((Modifier::Ctrl, is_pressed)), 
        espanso_detect::event::Key::Meta => Some((Modifier::Meta, is_pressed)), 
        espanso_detect::event::Key::Shift => Some((Modifier::Shift, is_pressed)),
        _ => None 
      }
    }
    _ => None,
  }
}

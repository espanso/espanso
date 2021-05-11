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

use std::{
  time::{Duration, Instant},
};

use log::{trace, warn};

use super::super::Middleware;
use crate::engine::event::{Event, EventType};

/// Maximum time to wait for modifiers being released before
/// giving up.
const MODIFIER_DELAY_TIMEOUT: Duration = Duration::from_secs(3);

pub trait ModifierStatusProvider {
  fn is_any_conflicting_modifier_pressed(&self) -> bool;
}

/// This middleware is used to delay the injection of text until
/// all modifiers have been released. This is needed as otherwise,
/// injections might misbehave as pressed modifiers might alter
/// the keys being injected.
pub struct DelayForModifierReleaseMiddleware<'a> {
  provider: &'a dyn ModifierStatusProvider,
}

impl <'a> DelayForModifierReleaseMiddleware<'a> {
  pub fn new(provider: &'a dyn ModifierStatusProvider) -> Self {
    Self {
      provider
    }
  }
}

impl <'a> Middleware for DelayForModifierReleaseMiddleware<'a> {
  fn name(&self) -> &'static str {
    "delay_modifiers"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    if is_injection_event(&event.etype) {
      let start = Instant::now();
      while self.provider.is_any_conflicting_modifier_pressed() {
        if Instant::now().duration_since(start) > MODIFIER_DELAY_TIMEOUT {
          warn!("injection delay has timed out, please release the modifier keys (SHIFT, CTRL, ALT, CMD) to trigger an expansion");
          break;
        }

        // TODO: here we might show a popup window to tell the users to release those keys

        trace!("delaying injection event as some modifiers are pressed");
        std::thread::sleep(Duration::from_millis(100));
      }
    }

    event
  }
}

fn is_injection_event(event_type: &EventType) -> bool {
  match event_type {
    EventType::TriggerCompensation(_) => true,
    EventType::CursorHintCompensation(_) => true,
    EventType::KeySequenceInject(_) => true,
    EventType::TextInject(_) => true,
    _ => false,
  }
}

// TODO: test

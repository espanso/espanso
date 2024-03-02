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
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use log::warn;

use espanso_engine::process::ModifierStatusProvider;

/// This duration represents the maximum length for which a pressed modifier
/// event is considered valid. This is useful when the "release" event is
/// lost for whatever reason, so that espanso becomes eventually consistent
/// after a while.
const MAXIMUM_MODIFIERS_PRESS_TIME_RECORD: Duration = Duration::from_secs(30);

const CONFLICTING_MODIFIERS: &[Modifier] = &[
    Modifier::Ctrl,
    Modifier::Alt,
    Modifier::Meta,
    Modifier::Shift,
];

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    Meta,
}

#[derive(Clone)]
pub struct ModifierStateStore {
    state: Arc<Mutex<ModifiersState>>,
}

impl ModifierStateStore {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ModifiersState::default())),
        }
    }

    pub fn is_any_conflicting_modifier_pressed(&self) -> bool {
        let mut state = self.state.lock().expect("unable to obtain modifier state");
        let mut is_any_modifier_pressed = false;
        for (modifier, status) in &mut state.modifiers {
            if status.is_outdated() {
                warn!(
                    "detected outdated modifier records for {:?}, releasing the state",
                    modifier
                );
                status.release();
            }

            if status.is_pressed() && CONFLICTING_MODIFIERS.contains(modifier) {
                is_any_modifier_pressed = true;
            }
        }
        is_any_modifier_pressed
    }

    pub fn update_state(&self, modifier: Modifier, is_pressed: bool) {
        let mut state = self.state.lock().expect("unable to obtain modifier state");
        for (curr_modifier, status) in &mut state.modifiers {
            if curr_modifier == &modifier {
                if is_pressed {
                    status.press();
                } else {
                    status.release();
                }
                break;
            }
        }
    }

    pub fn clear_state(&self) {
        let mut state = self.state.lock().expect("unable to obtain modifier state");
        for (_, status) in &mut state.modifiers {
            status.release();
        }
    }
}

struct ModifiersState {
    modifiers: Vec<(Modifier, ModifierStatus)>,
}

impl Default for ModifiersState {
    fn default() -> Self {
        Self {
            modifiers: vec![
                (Modifier::Ctrl, ModifierStatus { pressed_at: None }),
                (Modifier::Alt, ModifierStatus { pressed_at: None }),
                (Modifier::Shift, ModifierStatus { pressed_at: None }),
                (Modifier::Meta, ModifierStatus { pressed_at: None }),
            ],
        }
    }
}

struct ModifierStatus {
    pressed_at: Option<Instant>,
}

impl ModifierStatus {
    fn is_pressed(&self) -> bool {
        self.pressed_at.is_some()
    }

    fn is_outdated(&self) -> bool {
        let now = Instant::now();
        if let Some(pressed_at) = self.pressed_at {
            now.duration_since(pressed_at) > MAXIMUM_MODIFIERS_PRESS_TIME_RECORD
        } else {
            false
        }
    }

    fn release(&mut self) {
        self.pressed_at = None;
    }

    fn press(&mut self) {
        self.pressed_at = Some(Instant::now());
    }
}

impl ModifierStatusProvider for ModifierStateStore {
    fn is_any_conflicting_modifier_pressed(&self) -> bool {
        self.is_any_conflicting_modifier_pressed()
    }
}

impl espanso_engine::process::ModifierStateProvider for ModifierStateStore {
    fn get_modifier_state(&self) -> espanso_engine::process::ModifierState {
        let mut state = self.state.lock().expect("unable to obtain modifier state");

        let mut is_ctrl_down = false;
        let mut is_alt_down = false;
        let mut is_meta_down = false;

        for (modifier, status) in &mut state.modifiers {
            if status.is_outdated() {
                warn!(
                    "detected outdated modifier records for {:?}, releasing the state",
                    modifier
                );
                status.release();
            }

            if status.is_pressed() {
                match modifier {
                    Modifier::Ctrl => {
                        is_ctrl_down = true;
                    }
                    Modifier::Alt => {
                        is_alt_down = true;
                    }
                    Modifier::Meta => {
                        is_meta_down = true;
                    }
                    Modifier::Shift => {}
                }
            }
        }

        espanso_engine::process::ModifierState {
            is_ctrl_down,
            is_alt_down,
            is_meta_down,
        }
    }
}

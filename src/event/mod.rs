/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

pub(crate) mod manager;

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Event {
    Action(ActionType),
    Key(KeyEvent),
    System(SystemEvent),
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Noop = 0,
    Toggle = 1,
    Exit = 2,
    IconClick = 3,
    Enable = 4,
    Disable = 5,
    RestartWorker = 6,
    ExitWorker = 7,
}

impl From<i32> for ActionType {
    fn from(id: i32) -> Self {
        match id {
            1 => ActionType::Toggle,
            2 => ActionType::Exit,
            3 => ActionType::IconClick,
            4 => ActionType::Enable,
            5 => ActionType::Disable,
            6 => ActionType::RestartWorker,
            7 => ActionType::ExitWorker,
            _ => ActionType::Noop,
        }
    }
}

#[derive(Debug, Clone)]
pub enum KeyEvent {
    Char(String),
    Modifier(KeyModifier),
    Other,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyModifier {
    CTRL,
    SHIFT,
    ALT,
    META,
    BACKSPACE,
    OFF,

    // These are specific variants of the ones above. See issue: #117
    // https://github.com/federico-terzi/espanso/issues/117
    LEFT_CTRL,
    RIGHT_CTRL,
    LEFT_ALT,
    RIGHT_ALT,
    LEFT_META,
    RIGHT_META,
    LEFT_SHIFT,
    RIGHT_SHIFT,

    // Special cases, should not be used in config
    CAPS_LOCK,
}

impl KeyModifier {
    /// This function is used to compare KeyModifiers, considering the relations between
    /// the generic modifier and the specific left/right variant
    /// For example, CTRL will match with CTRL, LEFT_CTRL and RIGHT_CTRL;
    /// but LEFT_CTRL will only match will LEFT_CTRL
    pub fn shallow_equals(current: &KeyModifier, config: &KeyModifier) -> bool {
        use KeyModifier::*;

        match config {
            KeyModifier::CTRL => {
                current == &LEFT_CTRL || current == &RIGHT_CTRL || current == &CTRL
            }
            KeyModifier::SHIFT => {
                current == &LEFT_SHIFT || current == &RIGHT_SHIFT || current == &SHIFT
            }
            KeyModifier::ALT => current == &LEFT_ALT || current == &RIGHT_ALT || current == &ALT,
            KeyModifier::META => {
                current == &LEFT_META || current == &RIGHT_META || current == &META
            }
            KeyModifier::BACKSPACE => current == &BACKSPACE,
            KeyModifier::LEFT_CTRL => current == &LEFT_CTRL,
            KeyModifier::RIGHT_CTRL => current == &RIGHT_CTRL,
            KeyModifier::LEFT_ALT => current == &LEFT_ALT,
            KeyModifier::RIGHT_ALT => current == &RIGHT_ALT,
            KeyModifier::LEFT_META => current == &LEFT_META,
            KeyModifier::RIGHT_META => current == &RIGHT_META,
            KeyModifier::LEFT_SHIFT => current == &LEFT_SHIFT,
            KeyModifier::RIGHT_SHIFT => current == &RIGHT_SHIFT,
            _ => false,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SystemEvent {
    // MacOS specific
    SecureInputEnabled(String, String), // AppName, App Path
    SecureInputDisabled,

    // Notification
    NotifyRequest(String),
}

// Receivers

pub trait KeyEventReceiver {
    fn on_key_event(&self, e: KeyEvent);
}

pub trait ActionEventReceiver {
    fn on_action_event(&self, e: ActionType);
}

pub trait SystemEventReceiver {
    fn on_system_event(&self, e: SystemEvent);
}

// TESTS

#[cfg(test)]
mod tests {
    use super::KeyModifier::*;
    use super::*;

    #[test]
    fn test_shallow_equals_ctrl() {
        assert!(KeyModifier::shallow_equals(&CTRL, &CTRL));
        assert!(KeyModifier::shallow_equals(&LEFT_CTRL, &CTRL));
        assert!(KeyModifier::shallow_equals(&RIGHT_CTRL, &CTRL));

        assert!(!KeyModifier::shallow_equals(&CTRL, &LEFT_CTRL));
        assert!(!KeyModifier::shallow_equals(&CTRL, &RIGHT_CTRL));
    }

    #[test]
    fn test_shallow_equals_shift() {
        assert!(KeyModifier::shallow_equals(&SHIFT, &SHIFT));
        assert!(KeyModifier::shallow_equals(&LEFT_SHIFT, &SHIFT));
        assert!(KeyModifier::shallow_equals(&RIGHT_SHIFT, &SHIFT));

        assert!(!KeyModifier::shallow_equals(&SHIFT, &LEFT_SHIFT));
        assert!(!KeyModifier::shallow_equals(&SHIFT, &RIGHT_SHIFT));
    }

    #[test]
    fn test_shallow_equals_alt() {
        assert!(KeyModifier::shallow_equals(&ALT, &ALT));
        assert!(KeyModifier::shallow_equals(&LEFT_ALT, &ALT));
        assert!(KeyModifier::shallow_equals(&RIGHT_ALT, &ALT));

        assert!(!KeyModifier::shallow_equals(&ALT, &LEFT_ALT));
        assert!(!KeyModifier::shallow_equals(&ALT, &RIGHT_ALT));
    }

    #[test]
    fn test_shallow_equals_meta() {
        assert!(KeyModifier::shallow_equals(&META, &META));
        assert!(KeyModifier::shallow_equals(&LEFT_META, &META));
        assert!(KeyModifier::shallow_equals(&RIGHT_META, &META));

        assert!(!KeyModifier::shallow_equals(&META, &LEFT_META));
        assert!(!KeyModifier::shallow_equals(&META, &RIGHT_META));
    }

    #[test]
    fn test_shallow_equals_backspace() {
        assert!(KeyModifier::shallow_equals(&BACKSPACE, &BACKSPACE));
    }

    #[test]
    fn test_shallow_equals_off() {
        assert!(!KeyModifier::shallow_equals(&OFF, &CTRL));
        assert!(!KeyModifier::shallow_equals(&OFF, &ALT));
        assert!(!KeyModifier::shallow_equals(&OFF, &META));
        assert!(!KeyModifier::shallow_equals(&OFF, &SHIFT));
    }
}

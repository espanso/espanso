pub(crate) mod manager;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub enum Event {
    Action(ActionType),
    Key(KeyEvent)
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Noop = 0,
    Toggle = 1,
    Exit = 2,
    IconClick = 3,
    Enable = 4,
    Disable = 5,
}

impl From<i32> for ActionType {
    fn from(id: i32) -> Self {
        match id {
            1 => ActionType::Toggle,
            2 => ActionType::Exit,
            3 => ActionType::IconClick,
            4 => ActionType::Enable,
            5 => ActionType::Disable,
            _ => ActionType::Noop,
        }
    }
}

#[derive(Debug, Clone)]
pub enum KeyEvent {
    Char(char),
    Modifier(KeyModifier)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyModifier {
    CTRL,
    SHIFT,
    ALT,
    META,
    BACKSPACE,
}

impl Default for KeyModifier {
    fn default() -> Self {
        KeyModifier::ALT
    }
}

// Receivers

pub trait KeyEventReceiver {
    fn on_key_event(&self, e: KeyEvent);
}

pub trait ActionEventReceiver {
    fn on_action_event(&self, e: ActionType);
}
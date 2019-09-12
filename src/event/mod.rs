pub(crate) mod manager;

use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum Event {
    Action(ActionEvent),
    Key(KeyEvent)
}

#[derive(Debug)]
pub enum ActionEvent {
    IconClick,
    ContextMenuClick(i32)
}

#[derive(Debug)]
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
    fn on_action_event(&self, e: ActionEvent); // TODO: Action event
}
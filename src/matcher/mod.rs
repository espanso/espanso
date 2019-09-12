use std::sync::mpsc::Receiver;
use serde::{Serialize, Deserialize};
use crate::event::{KeyEvent, KeyModifier};
use crate::event::KeyEventReceiver;

pub(crate) mod scrolling;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Match {
    pub trigger: String,
    pub replace: String
}

pub trait MatchReceiver {
    fn on_match(&self, m: &Match);
    fn on_toggle(&self, status: bool);
}

pub trait Matcher : KeyEventReceiver {
    fn handle_char(&self, c: char);
    fn handle_modifier(&self, m: KeyModifier);
}

impl <M: Matcher> KeyEventReceiver for M {
    fn on_key_event(&self, e: KeyEvent) {
        match e {
            KeyEvent::Char(c) => {
                self.handle_char(c);
            },
            KeyEvent::Modifier(m) => {
                self.handle_modifier(m);
            },
        }
    }
}
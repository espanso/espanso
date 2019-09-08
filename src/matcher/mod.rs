use std::sync::mpsc::Receiver;
use serde::{Serialize, Deserialize};
use crate::keyboard::{KeyEvent, KeyModifier};

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

pub trait Matcher {
    fn handle_char(&self, c: char);
    fn handle_modifier(&self, m: KeyModifier);
    fn watch(&self, receiver: Receiver<KeyEvent>) {
        loop {
            match receiver.recv() {
                Ok(event) => {
                    match event {
                        KeyEvent::Char(c) => {
                            self.handle_char(c);
                        },
                        KeyEvent::Modifier(m) => {
                            self.handle_modifier(m);
                        },
                    }
                },
                Err(_) => panic!("Keyboard interceptor broke receiver stream."),
            }
        }
    }
}
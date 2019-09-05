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
}

pub trait Matcher<'a>: Send {
    fn handle_char(&'a self, c: char);
    fn handle_modifier(&'a self, m: KeyModifier);
    fn watch(&'a self, receiver: Receiver<KeyEvent>) {
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
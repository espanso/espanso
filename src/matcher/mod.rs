use std::sync::mpsc::Receiver;
use serde::{Serialize, Deserialize};

pub(crate) mod scrolling;

#[derive(Debug, Serialize, Deserialize)]
pub struct Match {
    pub trigger: String,
    pub replace: String
}

pub trait MatchReceiver {
    fn on_match(&self, m: &Match);
}

pub trait Matcher {
    fn handle_char(&mut self, c: char);
    fn watch(&mut self, receiver: &Receiver<char>) {
        loop {
            match receiver.recv() {
                Ok(c) => {
                    self.handle_char(c);
                },
                Err(_) => panic!("Keyboard interceptor broke receiver stream."),
            }
        }
    }
}
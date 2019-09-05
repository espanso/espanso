use std::sync::mpsc::Receiver;
use serde::{Serialize, Deserialize};

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
    fn watch(&'a self, receiver: Receiver<char>) {
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
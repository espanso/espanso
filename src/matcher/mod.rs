use std::sync::mpsc::Receiver;

pub(crate) mod scrolling;

pub struct Match {
    pub trigger: String,
    pub result: String
}

pub trait MatchReceiver {
    fn on_match(&self, m: Match);
}

pub trait Matcher {
    fn handle_char(&self, c: char);
    fn watch(&self, receiver: &Receiver<char>) {
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
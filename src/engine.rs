use crate::matcher::{Match, MatchReceiver};
use crate::keyboard::KeyboardSender;

pub struct Engine<S> where S: KeyboardSender {
    sender: S
}

impl <S> Engine<S> where S: KeyboardSender{
    pub fn new(sender: S) -> Engine<S> where S: KeyboardSender {
        Engine{sender}
    }
}

impl <S> MatchReceiver for Engine<S> where S: KeyboardSender{
    fn on_match(&self, m: &Match) {
        self.sender.delete_string(m.trigger.len() as i32);
        self.sender.send_string(m.replace.as_str());
    }
}
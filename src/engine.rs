use crate::matcher::{Match, MatchReceiver};
use crate::keyboard::KeyboardSender;
use crate::config::Configs;

pub struct Engine<S> where S: KeyboardSender {
    sender: S,
    configs: Configs,
}

impl <S> Engine<S> where S: KeyboardSender{
    pub fn new(sender: S, configs: Configs) -> Engine<S> where S: KeyboardSender {
        Engine{sender, configs }
    }
}

impl <S> MatchReceiver for Engine<S> where S: KeyboardSender{
    fn on_match(&self, m: &Match) {
        self.sender.delete_string(m.trigger.len() as i32);

        // Send the expected string. On linux, newlines are managed automatically
        // while on windows and macos, we need to emulate a Enter key press.

        if cfg!(target_os = "linux") {
            self.sender.send_string(m.replace.as_str());
        }else{
            // To handle newlines, substitute each "\n" char with an Enter key press.
            let splits = m.replace.lines();

            for (i, split) in splits.enumerate() {
                if i > 0 {
                    self.sender.send_enter();
                }

                self.sender.send_string(split);
            }
        }
    }
}
use crate::matcher::{Match, MatchReceiver};
use crate::keyboard::KeyboardSender;
use crate::config::Configs;
use crate::clipboard::ClipboardManager;
use std::sync::Arc;

pub struct Engine<S, C> where S: KeyboardSender, C: ClipboardManager {
    sender: S,
    clipboard_manager: Arc<C>,
    configs: Configs,
}

impl <S, C> Engine<S, C> where S: KeyboardSender, C: ClipboardManager{
    pub fn new(sender: S, clipboard_manager: Arc<C>, configs: Configs) -> Engine<S, C> where S: KeyboardSender, C: ClipboardManager {
        Engine{sender, clipboard_manager, configs }
    }
}

impl <S, C> MatchReceiver for Engine<S, C> where S: KeyboardSender, C: ClipboardManager{
    fn on_match(&self, m: &Match) {
        self.sender.delete_string(m.trigger.len() as i32);

        // Send the expected string. On linux, newlines are managed automatically
        // while on windows and macos, we need to emulate a Enter key press.

        if cfg!(target_os = "linux") {
            self.clipboard_manager.set_clipboard(m.replace.as_str());
            self.sender.trigger_paste();
            //self.sender.send_string(m.replace.as_str());
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
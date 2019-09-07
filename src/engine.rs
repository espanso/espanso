use crate::matcher::{Match, MatchReceiver};
use crate::keyboard::KeyboardSender;
use crate::config::ConfigManager;
use crate::config::BackendType;
use crate::clipboard::ClipboardManager;

pub struct Engine<'a, S: KeyboardSender, C: ClipboardManager, M: ConfigManager> {
    sender: S,
    clipboard_manager: &'a C,
    config_manager: &'a M,
}

impl <'a, S: KeyboardSender, C: ClipboardManager, M: ConfigManager> Engine<'a, S, C, M> {
    pub fn new<'b>(sender: S, clipboard_manager: &'b C, config_manager: &'b M) -> Engine<'b, S, C, M> {
        Engine{sender, clipboard_manager, config_manager }
    }
}

impl <'a, S: KeyboardSender, C: ClipboardManager, M: ConfigManager> MatchReceiver for Engine<'a, S, C, M>{
    fn on_match(&self, m: &Match) {
        let config = self.config_manager.default_config();

        self.sender.delete_string(m.trigger.len() as i32);

        match config.backend {
            BackendType::Inject => {
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
            },
            BackendType::Clipboard => {
                self.clipboard_manager.set_clipboard(m.replace.as_str());
                self.sender.trigger_paste();
            },
        }
    }
}
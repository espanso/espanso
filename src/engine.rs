use crate::matcher::{Match, MatchReceiver};
use crate::keyboard::KeyboardSender;
use crate::config::ConfigManager;
use crate::config::BackendType;
use crate::clipboard::ClipboardManager;
use log::{info};
use crate::ui::{UIManager, MenuItem, MenuItemType};
use crate::event::{ActionEventReceiver, Event, ActionEvent};
use std::cell::RefCell;

pub struct Engine<'a, S: KeyboardSender, C: ClipboardManager, M: ConfigManager<'a>,
                  U: UIManager> {
    sender: S,
    clipboard_manager: &'a C,
    config_manager: &'a M,
    ui_manager: &'a U,
    enabled: RefCell<bool>,
}

impl <'a, S: KeyboardSender, C: ClipboardManager, M: ConfigManager<'a>, U: UIManager>
    Engine<'a, S, C, M, U> {
    pub fn new(sender: S, clipboard_manager: &'a C, config_manager: &'a M, ui_manager: &'a U) -> Engine<'a, S, C, M, U> {
        let enabled = RefCell::new(true);
        Engine{sender, clipboard_manager, config_manager, ui_manager, enabled }
    }

    fn build_menu(&self) -> Vec<MenuItem> {
        let mut menu = Vec::new();

        let enabled = self.enabled.borrow();
        let toggle_text = if *enabled {
            "Disable"
        }else{
            "Enable"
        }.to_owned();
        menu.push(MenuItem{
            item_type: MenuItemType::Button,
            item_name: toggle_text,
            item_id: 2,
        });

        menu.push(MenuItem{
            item_type: MenuItemType::Separator,
            item_name: "".to_owned(),
            item_id: 999,
        });

        menu.push(MenuItem{
            item_type: MenuItemType::Button,
            item_name: "Exit".to_owned(),
            item_id: 1,
        });

        menu
    }
}

impl <'a, S: KeyboardSender, C: ClipboardManager, M: ConfigManager<'a>, U: UIManager>
    MatchReceiver for Engine<'a, S, C, M, U>{

    fn on_match(&self, m: &Match) {
        let config = self.config_manager.active_config();

        if config.disabled {
            return;
        }

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

    fn on_toggle(&self, status: bool) {
        let message = if status {
            "espanso enabled"
        }else{
            "espanso disabled"
        };

        info!("Toggled: {}", message);

        let mut enabled_ref = self.enabled.borrow_mut();
        *enabled_ref = status;

        self.ui_manager.notify(message);
    }
}

impl <'a, S: KeyboardSender, C: ClipboardManager,
    M: ConfigManager<'a>, U: UIManager> ActionEventReceiver for Engine<'a, S, C, M, U>{

    fn on_action_event(&self, e: ActionEvent) {
        match e {
            ActionEvent::IconClick => {
                self.ui_manager.show_menu(self.build_menu());
            },
            ActionEvent::ContextMenuClick(id) => {
                println!("{}", id);
            }
        }
    }
}
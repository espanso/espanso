use crate::matcher::{Match, MatchReceiver};
use crate::keyboard::KeyboardManager;
use crate::config::ConfigManager;
use crate::config::BackendType;
use crate::clipboard::ClipboardManager;
use log::{info};
use crate::ui::{UIManager, MenuItem, MenuItemType};
use crate::event::{ActionEventReceiver, ActionType};
use std::cell::RefCell;
use std::process::exit;

pub struct Engine<'a, S: KeyboardManager, C: ClipboardManager, M: ConfigManager<'a>,
                  U: UIManager> {
    keyboard_manager: &'a S,
    clipboard_manager: &'a C,
    config_manager: &'a M,
    ui_manager: &'a U,
    enabled: RefCell<bool>,
}

impl <'a, S: KeyboardManager, C: ClipboardManager, M: ConfigManager<'a>, U: UIManager>
    Engine<'a, S, C, M, U> {
    pub fn new(keyboard_manager: &'a S, clipboard_manager: &'a C, config_manager: &'a M, ui_manager: &'a U) -> Engine<'a, S, C, M, U> {
        let enabled = RefCell::new(true);
        Engine{keyboard_manager, clipboard_manager, config_manager, ui_manager, enabled }
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
            item_id: ActionType::Toggle as i32,
        });

        menu.push(MenuItem{
            item_type: MenuItemType::Separator,
            item_name: "".to_owned(),
            item_id: 999,
        });

        menu.push(MenuItem{
            item_type: MenuItemType::Button,
            item_name: "Exit".to_owned(),
            item_id: ActionType::Exit as i32,
        });

        menu
    }
}

impl <'a, S: KeyboardManager, C: ClipboardManager, M: ConfigManager<'a>, U: UIManager>
    MatchReceiver for Engine<'a, S, C, M, U>{

    fn on_match(&self, m: &Match) {
        let config = self.config_manager.active_config();

        if config.disabled {
            return;
        }

        self.keyboard_manager.delete_string(m.trigger.len() as i32);

        match config.backend {
            BackendType::Inject => {
                // Send the expected string. On linux, newlines are managed automatically
                // while on windows and macos, we need to emulate a Enter key press.

                if cfg!(target_os = "linux") {
                    self.keyboard_manager.send_string(m.replace.as_str());
                }else{
                    // To handle newlines, substitute each "\n" char with an Enter key press.
                    let splits = m.replace.lines();

                    for (i, split) in splits.enumerate() {
                        if i > 0 {
                            self.keyboard_manager.send_enter();
                        }

                        self.keyboard_manager.send_string(split);
                    }
                }
            },
            BackendType::Clipboard => {
                self.clipboard_manager.set_clipboard(m.replace.as_str());
                self.keyboard_manager.trigger_paste();
            },
        }
    }

    fn on_enable_update(&self, status: bool) {
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

impl <'a, S: KeyboardManager, C: ClipboardManager,
    M: ConfigManager<'a>, U: UIManager> ActionEventReceiver for Engine<'a, S, C, M, U>{

    fn on_action_event(&self, e: ActionType) {
        match e {
            ActionType::IconClick => {
                self.ui_manager.show_menu(self.build_menu());
            },
            ActionType::Exit => {
                info!("Terminating espanso.");
                exit(0);
            },
            _ => {}
        }
    }
}
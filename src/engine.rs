/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::clipboard::ClipboardManager;
use crate::config::BackendType;
use crate::config::ConfigManager;
use crate::event::{ActionEventReceiver, ActionType, SystemEvent, SystemEventReceiver};
use crate::keyboard::KeyboardManager;
use crate::matcher::{Match, MatchReceiver};
use crate::protocol::{send_command_or_warn, IPCCommand, Service};
use crate::render::{RenderResult, Renderer};
use crate::ui::{MenuItem, MenuItemType, UIManager};
use log::{debug, error, info, warn};
use regex::Regex;
use std::cell::RefCell;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Release;
use std::sync::Arc;

pub struct Engine<
    'a,
    S: KeyboardManager,
    C: ClipboardManager,
    M: ConfigManager<'a>,
    U: UIManager,
    R: Renderer,
> {
    keyboard_manager: &'a S,
    clipboard_manager: &'a C,
    config_manager: &'a M,
    ui_manager: &'a U,
    renderer: &'a R,
    is_injecting: Arc<AtomicBool>,

    enabled: RefCell<bool>,
}

impl<
        'a,
        S: KeyboardManager,
        C: ClipboardManager,
        M: ConfigManager<'a>,
        U: UIManager,
        R: Renderer,
    > Engine<'a, S, C, M, U, R>
{
    pub fn new(
        keyboard_manager: &'a S,
        clipboard_manager: &'a C,
        config_manager: &'a M,
        ui_manager: &'a U,
        renderer: &'a R,
        is_injecting: Arc<AtomicBool>,
    ) -> Engine<'a, S, C, M, U, R> {
        let enabled = RefCell::new(true);

        Engine {
            keyboard_manager,
            clipboard_manager,
            config_manager,
            ui_manager,
            renderer,
            is_injecting,
            enabled,
        }
    }

    fn build_menu(&self) -> Vec<MenuItem> {
        let mut menu = Vec::new();

        let enabled = self.enabled.borrow();
        let toggle_text = if *enabled { "Disable" } else { "Enable" }.to_owned();
        menu.push(MenuItem {
            item_type: MenuItemType::Button,
            item_name: toggle_text,
            item_id: ActionType::Toggle as i32,
        });

        menu.push(MenuItem {
            item_type: MenuItemType::Separator,
            item_name: "".to_owned(),
            item_id: 998,
        });

        menu.push(MenuItem {
            item_type: MenuItemType::Button,
            item_name: "Reload configs".to_owned(),
            item_id: ActionType::RestartWorker as i32,
        });

        menu.push(MenuItem {
            item_type: MenuItemType::Separator,
            item_name: "".to_owned(),
            item_id: 999,
        });

        menu.push(MenuItem {
            item_type: MenuItemType::Button,
            item_name: "Exit espanso".to_owned(),
            item_id: ActionType::Exit as i32,
        });

        menu
    }

    fn return_content_if_preserve_clipboard_is_enabled(&self) -> Option<String> {
        // If the preserve_clipboard option is enabled, first save the current
        // clipboard content in order to restore it later.
        if self.config_manager.default_config().preserve_clipboard {
            match self.clipboard_manager.get_clipboard() {
                Some(clipboard) => Some(clipboard),
                None => None,
            }
        } else {
            None
        }
    }

    fn find_match_by_trigger(&self, trigger: &str) -> Option<Match> {
        let config = self.config_manager.active_config();

        if let Some(m) = config
            .matches
            .iter()
            .find(|m| m.triggers.iter().any(|t| t == trigger))
        {
            Some(m.clone())
        } else {
            None
        }
    }

    fn inject_match(
        &self,
        m: &Match,
        trailing_separator: Option<char>,
        trigger_offset: usize,
        skip_delete: bool,
    ) {
        let config = self.config_manager.active_config();

        if !config.enable_active {
            return;
        }

        // Block espanso from reinterpreting its own actions
        self.is_injecting.store(true, Release);

        let char_count = if trailing_separator.is_none() {
            m.triggers[trigger_offset].chars().count() as i32
        } else {
            m.triggers[trigger_offset].chars().count() as i32 + 1 // Count also the separator
        };

        if !skip_delete {
            self.keyboard_manager.delete_string(&config, char_count);
        }

        let mut previous_clipboard_content: Option<String> = None;

        let rendered = self
            .renderer
            .render_match(m, trigger_offset, config, vec![]);

        match rendered {
            RenderResult::Text(mut target_string) => {
                // If a trailing separator was counted in the match, add it back to the target string
                if let Some(trailing_separator) = trailing_separator {
                    if trailing_separator == '\r' {
                        // If the trailing separator is a carriage return,
                        target_string.push('\n'); // convert it to new line
                    } else {
                        target_string.push(trailing_separator);
                    }
                }

                // Convert Windows style newlines into unix styles
                target_string = target_string.replace("\r\n", "\n");

                // Calculate cursor rewind moves if a Cursor Hint is present
                let index = target_string.find("$|$");
                let cursor_rewind = if let Some(index) = index {
                    // Convert the byte index to a char index
                    let char_str = &target_string[0..index];
                    let char_index = char_str.chars().count();
                    let total_size = target_string.chars().count();

                    // Remove the $|$ placeholder
                    target_string = target_string.replace("$|$", "");

                    // Calculate the amount of rewind moves needed (LEFT ARROW).
                    // Subtract also 3, equal to the number of chars of the placeholder "$|$"
                    let moves = (total_size - char_index - 3) as i32;
                    Some(moves)
                } else {
                    None
                };

                let backend = if m.force_clipboard {
                    &BackendType::Clipboard
                } else if config.backend == BackendType::Auto {
                    if cfg!(target_os = "linux") {
                        let all_ascii = target_string.chars().all(|c| c.is_ascii());
                        if all_ascii {
                            debug!(
                                "All elements of the replacement are ascii, using Inject backend"
                            );
                            &BackendType::Inject
                        } else {
                            debug!("There are non-ascii characters, using Clipboard backend");
                            &BackendType::Clipboard
                        }
                    } else {
                        &BackendType::Inject
                    }
                } else {
                    &config.backend
                };

                match backend {
                    BackendType::Inject => {
                        // To handle newlines, substitute each "\n" char with an Enter key press.
                        let splits = target_string.split('\n');

                        for (i, split) in splits.enumerate() {
                            if i > 0 {
                                self.keyboard_manager.send_enter(&config);
                            }

                            self.keyboard_manager.send_string(&config, split);
                        }
                    }
                    BackendType::Clipboard => {
                        // If the preserve_clipboard option is enabled, save the current
                        // clipboard content to restore it later.
                        previous_clipboard_content =
                            self.return_content_if_preserve_clipboard_is_enabled();

                        self.clipboard_manager.set_clipboard(&target_string);
                        self.keyboard_manager.trigger_paste(&config);
                    }
                    _ => {
                        error!("Unsupported backend type evaluation.");
                        return;
                    }
                }

                if let Some(moves) = cursor_rewind {
                    // Simulate left arrow key presses to bring the cursor into the desired position
                    self.keyboard_manager.move_cursor_left(&config, moves);
                }
            }
            RenderResult::Image(image_path) => {
                // If the preserve_clipboard option is enabled, save the current
                // clipboard content to restore it later.
                previous_clipboard_content = self.return_content_if_preserve_clipboard_is_enabled();

                self.clipboard_manager.set_clipboard_image(&image_path);
                self.keyboard_manager.trigger_paste(&config);
            }
            RenderResult::Error => {
                error!("Could not render match: {}", m.triggers[trigger_offset]);
            }
        }

        // Restore previous clipboard content
        if let Some(previous_clipboard_content) = previous_clipboard_content {
            // Sometimes an expansion gets overwritten before pasting by the previous content
            // A delay is needed to mitigate the problem
            std::thread::sleep(std::time::Duration::from_millis(
                config.restore_clipboard_delay as u64,
            ));

            self.clipboard_manager
                .set_clipboard(&previous_clipboard_content);
        }

        // Re-allow espanso to interpret actions
        self.is_injecting.store(false, Release);
    }
}

lazy_static! {
    static ref VAR_REGEX: Regex = Regex::new("\\{\\{\\s*(?P<name>\\w+)\\s*\\}\\}").unwrap();
}

impl<
        'a,
        S: KeyboardManager,
        C: ClipboardManager,
        M: ConfigManager<'a>,
        U: UIManager,
        R: Renderer,
    > MatchReceiver for Engine<'a, S, C, M, U, R>
{
    fn on_match(&self, m: &Match, trailing_separator: Option<char>, trigger_offset: usize) {
        self.inject_match(m, trailing_separator, trigger_offset, false);
    }

    fn on_enable_update(&self, status: bool) {
        let message = if status {
            "espanso enabled"
        } else {
            "espanso disabled"
        };

        info!("Toggled: {}", message);

        let mut enabled_ref = self.enabled.borrow_mut();
        *enabled_ref = status;

        let config = self.config_manager.default_config();

        if config.show_notifications {
            self.ui_manager.notify(message);
        }
    }

    fn on_passive(&self) {
        let config = self.config_manager.active_config();

        if !config.enable_passive {
            return;
        }

        // Block espanso from reinterpreting its own actions
        self.is_injecting.store(true, Release);

        // In order to avoid pasting previous clipboard contents, we need to check if
        // a new clipboard was effectively copied.
        // See issue: https://github.com/federico-terzi/espanso/issues/213
        let previous_clipboard = self.clipboard_manager.get_clipboard();

        // Sleep for a while, giving time to effectively copy the text
        std::thread::sleep(std::time::Duration::from_millis(100)); // TODO: avoid hardcoding

        // Trigger a copy shortcut to transfer the content of the selection to the clipboard
        self.keyboard_manager.trigger_copy(&config);

        // Sleep for a while, giving time to effectively copy the text
        std::thread::sleep(std::time::Duration::from_millis(100)); // TODO: avoid hardcoding

        // Then get the text from the clipboard and render the match output
        let clipboard = self.clipboard_manager.get_clipboard();

        if let Some(clipboard) = clipboard {
            // Don't expand empty clipboards, as usually they are the result of an empty passive selection
            if clipboard.trim().is_empty() {
                info!("Avoiding passive expansion, as the user didn't select anything");
            } else {
                if let Some(previous_content) = previous_clipboard {
                    // Because of issue #213, we need to make sure the user selected something.
                    if clipboard == previous_content {
                        info!("Avoiding passive expansion, as the user didn't select anything");
                    } else {
                        info!("Passive mode activated");

                        let rendered = self.renderer.render_passive(&clipboard, &config);

                        match rendered {
                            RenderResult::Text(payload) => {
                                // Paste back the result in the field
                                self.clipboard_manager.set_clipboard(&payload);

                                std::thread::sleep(std::time::Duration::from_millis(100)); // TODO: avoid hardcoding
                                self.keyboard_manager.trigger_paste(&config);
                            }
                            _ => warn!("Cannot expand passive match"),
                        }
                    }
                }
            }
        }

        // Re-allow espanso to interpret actions
        self.is_injecting.store(false, Release);
    }
}

impl<
        'a,
        S: KeyboardManager,
        C: ClipboardManager,
        M: ConfigManager<'a>,
        U: UIManager,
        R: Renderer,
    > ActionEventReceiver for Engine<'a, S, C, M, U, R>
{
    fn on_action_event(&self, e: ActionType) {
        let config = self.config_manager.default_config();
        match e {
            ActionType::IconClick => {
                self.ui_manager.show_menu(self.build_menu());
            }
            ActionType::ExitWorker => {
                info!("terminating worker process");
                self.ui_manager.cleanup();
                exit(0);
            }
            ActionType::Exit => {
                send_command_or_warn(Service::Daemon, config.clone(), IPCCommand::exit());
            }
            ActionType::RestartWorker => {
                send_command_or_warn(
                    Service::Daemon,
                    config.clone(),
                    IPCCommand::restart_worker(),
                );
            }
            _ => {}
        }
    }
}

impl<
        'a,
        S: KeyboardManager,
        C: ClipboardManager,
        M: ConfigManager<'a>,
        U: UIManager,
        R: Renderer,
    > SystemEventReceiver for Engine<'a, S, C, M, U, R>
{
    fn on_system_event(&self, e: SystemEvent) {
        match e {
            // MacOS specific
            SystemEvent::SecureInputEnabled(app_name, path) => {
                info!("SecureInput has been acquired by {}, preventing espanso from working correctly. Full path: {}", app_name, path);

                let config = self.config_manager.default_config();
                if config.secure_input_notification && config.show_notifications {
                    self.ui_manager.notify_delay(&format!("{} has activated SecureInput. Espanso won't work until you disable it.", app_name), 5000);
                }
            }
            SystemEvent::SecureInputDisabled => {
                info!("SecureInput has been disabled.");
            }
            SystemEvent::NotifyRequest(message) => {
                let config = self.config_manager.default_config();
                if config.show_notifications {
                    self.ui_manager.notify(&message);
                }
            }
            SystemEvent::Trigger(trigger) => {
                let m = self.find_match_by_trigger(&trigger);
                match m {
                    Some(m) => {
                        self.inject_match(&m, None, 0, true);
                    }
                    None => warn!("No match found with trigger: {}", trigger),
                }
            }
        }
    }
}

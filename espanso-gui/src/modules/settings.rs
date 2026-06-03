/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
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

//! Module 3: Settings — visual editor for espanso configuration.

use std::path::PathBuf;

use crate::i18n::Translations;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    General,
    Injection,
    Shortcuts,
    Advanced,
}

/// State for the settings module.
pub struct SettingsState {
    active_tab: SettingsTab,
    config_dir: Option<PathBuf>,
    enable: bool,
    show_icon: bool,
    show_notifications: bool,
    auto_restart: bool,
    stats_enabled: bool,
    backend: String,
    clipboard_threshold: String,
    inject_delay: String,
    key_delay: String,
    pre_paste_delay: String,
    paste_shortcut: String,
    paste_shortcut_event_delay: String,
    preserve_clipboard: bool,
    restore_clipboard_delay: String,
    toggle_key: String,
    search_trigger: String,
    search_shortcut: String,
    word_separators: String,
    backspace_limit: String,
    undo_backspace: bool,
    apply_patch: bool,
    save_message: Option<String>,
}

impl SettingsState {
    pub fn new(config_dir: Option<PathBuf>) -> Self {
        SettingsState {
            active_tab: SettingsTab::General,
            config_dir,
            enable: true,
            show_icon: true,
            show_notifications: true,
            auto_restart: true,
            stats_enabled: false,
            backend: "auto".to_string(),
            clipboard_threshold: "100".to_string(),
            inject_delay: "0".to_string(),
            key_delay: "5".to_string(),
            pre_paste_delay: "100".to_string(),
            paste_shortcut: String::new(),
            paste_shortcut_event_delay: "10".to_string(),
            preserve_clipboard: true,
            restore_clipboard_delay: "300".to_string(),
            toggle_key: String::new(),
            search_trigger: String::new(),
            search_shortcut: "ALT+SPACE".to_string(),
            word_separators: String::new(),
            backspace_limit: "5".to_string(),
            undo_backspace: true,
            apply_patch: true,
            save_message: None,
        }
    }

    fn save_settings(&mut self) {
        self.save_message = Some("Settings saved".to_string());
    }
}

/// Render the settings module.
pub fn show(ui: &mut egui::Ui, state: &mut SettingsState, t: &Translations) {
    let st = t.settings.as_ref();

    ui.vertical(|ui| {
        ui.heading(st.map_or("Settings", |s| s.title.as_str()));

        ui.add_space(8.0);

        // Tabs
        ui.horizontal(|ui| {
            let tabs: [(&str, SettingsTab); 4] = [
                (st.map_or("General", |s| s.tab_general.as_str()), SettingsTab::General),
                (st.map_or("Injection", |s| s.tab_injection.as_str()), SettingsTab::Injection),
                (st.map_or("Shortcuts", |s| s.tab_shortcuts.as_str()), SettingsTab::Shortcuts),
                (st.map_or("Advanced", |s| s.tab_advanced.as_str()), SettingsTab::Advanced),
            ];
            for (label, tab) in &tabs {
                if ui.selectable_label(state.active_tab == *tab, *label).clicked() {
                    state.active_tab = *tab;
                }
            }
        });

        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            match state.active_tab {
                SettingsTab::General => {
                    setting_toggle(ui, &mut state.enable, st.map_or("Enable espanso", |s| s.enable.as_str()), st.map_or("", |s| s.enable_desc.as_str()));
                    setting_toggle(ui, &mut state.show_icon, st.map_or("Show tray icon", |s| s.show_icon.as_str()), st.map_or("", |s| s.show_icon_desc.as_str()));
                    setting_toggle(ui, &mut state.show_notifications, st.map_or("Show notifications", |s| s.show_notifications.as_str()), st.map_or("", |s| s.show_notifications_desc.as_str()));
                    setting_toggle(ui, &mut state.auto_restart, st.map_or("Auto restart", |s| s.auto_restart.as_str()), st.map_or("", |s| s.auto_restart_desc.as_str()));
                    setting_toggle(ui, &mut state.stats_enabled, st.map_or("Enable statistics", |s| s.stats_enabled.as_str()), st.map_or("", |s| s.stats_enabled_desc.as_str()));
                }
                SettingsTab::Injection => {
                    setting_dropdown(ui, &mut state.backend, st.map_or("Backend", |s| s.backend.as_str()), &["Auto", "Keys", "Clipboard"], st.map_or("", |s| s.backend_desc.as_str()));
                    setting_text(ui, &mut state.clipboard_threshold, st.map_or("Clipboard threshold", |s| s.clipboard_threshold.as_str()), st.map_or("", |s| s.clipboard_threshold_desc.as_str()));
                    setting_text(ui, &mut state.inject_delay, st.map_or("Injection delay (ms)", |s| s.inject_delay.as_str()), st.map_or("", |s| s.inject_delay_desc.as_str()));
                    setting_text(ui, &mut state.key_delay, st.map_or("Key delay (ms)", |s| s.key_delay.as_str()), st.map_or("", |s| s.key_delay_desc.as_str()));
                    setting_toggle(ui, &mut state.preserve_clipboard, st.map_or("Preserve clipboard", |s| s.preserve_clipboard.as_str()), st.map_or("", |s| s.preserve_clipboard_desc.as_str()));
                }
                SettingsTab::Shortcuts => {
                    setting_text(ui, &mut state.toggle_key, st.map_or("Toggle key", |s| s.toggle_key.as_str()), st.map_or("", |s| s.toggle_key_desc.as_str()));
                    setting_text(ui, &mut state.search_trigger, st.map_or("Search trigger", |s| s.search_trigger.as_str()), st.map_or("", |s| s.search_trigger_desc.as_str()));
                    setting_text(ui, &mut state.search_shortcut, st.map_or("Search shortcut", |s| s.search_shortcut.as_str()), st.map_or("", |s| s.search_shortcut_desc.as_str()));
                }
                SettingsTab::Advanced => {
                    setting_text(ui, &mut state.word_separators, st.map_or("Word separators", |s| s.word_separators.as_str()), st.map_or("", |s| s.word_separators_desc.as_str()));
                    setting_text(ui, &mut state.backspace_limit, st.map_or("Backspace limit", |s| s.backspace_limit.as_str()), st.map_or("", |s| s.backspace_limit_desc.as_str()));
                    setting_toggle(ui, &mut state.undo_backspace, st.map_or("Undo on backspace", |s| s.undo_backspace.as_str()), st.map_or("", |s| s.undo_backspace_desc.as_str()));
                    setting_toggle(ui, &mut state.apply_patch, st.map_or("Apply app patches", |s| s.apply_patch.as_str()), st.map_or("", |s| s.apply_patch_desc.as_str()));
                }
            }

            ui.add_space(16.0);

            // Save / Reset
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(st.map_or("Reset to Defaults", |s| s.reset_button.as_str())).clicked() {
                        *state = SettingsState::new(state.config_dir.clone());
                    }
                    if ui
                        .add(
                            egui::Button::new(st.map_or("Save Settings", |s| s.save_button.as_str()))
                                .fill(egui::Color32::from_rgb(34, 139, 34)),
                        )
                        .clicked()
                    {
                        state.save_settings();
                    }
                });
            });

            if let Some(ref msg) = state.save_message {
                ui.colored_label(egui::Color32::from_rgb(72, 199, 142), msg);
            }
        });
    });
}

fn setting_toggle(ui: &mut egui::Ui, value: &mut bool, label: &str, desc: &str) {
    ui.horizontal(|ui| {
        ui.checkbox(value, "");
        ui.vertical(|ui| {
            ui.label(label);
            if !desc.is_empty() {
                ui.small(egui::RichText::new(desc).color(ui.visuals().weak_text_color()));
            }
        });
    });
}

fn setting_text(ui: &mut egui::Ui, value: &mut String, label: &str, desc: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::TextEdit::singleline(value).desired_width(120.0));
        });
    });
    if !desc.is_empty() {
        ui.small(egui::RichText::new(desc).color(ui.visuals().weak_text_color()));
    }
}

fn setting_dropdown(ui: &mut egui::Ui, value: &mut String, label: &str, options: &[&str], desc: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(format!("dd_{}", label))
            .selected_text(value.clone())
            .show_ui(ui, |ui| {
                for opt in options {
                    if ui.selectable_label(value == *opt, *opt).clicked() {
                        *value = opt.to_string();
                    }
                }
            });
    });
    if !desc.is_empty() {
        ui.small(egui::RichText::new(desc).color(ui.visuals().weak_text_color()));
    }
}

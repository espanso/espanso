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

//! Module 3: Settings — visual editor with YAML read/write persistence.

use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_norway::Mapping;

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

    // All settings fields (mirrors YAMLConfig from espanso-config)
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
    emulate_alt_codes: bool,
    max_form_width: String,
    max_form_height: String,

    save_message: Option<String>,
}

impl SettingsState {
    pub fn new(config_dir: Option<PathBuf>) -> Self {
        let mut state = SettingsState {
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
            emulate_alt_codes: true,
            max_form_width: "700".to_string(),
            max_form_height: "500".to_string(),
            save_message: None,
        };
        state.load_from_config();
        state
    }

    /// Load settings from default.yml into the state struct.
    fn load_from_config(&mut self) {
        let config_path = match &self.config_dir {
            Some(d) => d.join("config").join("default.yml"),
            None => return,
        };

        if !config_path.exists() {
            return;
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        // Parse as a generic YAML mapping to extract known keys
        let mapping: Result<Mapping, _> = serde_norway::from_str(&content);
        let mapping = match mapping {
            Ok(m) => m,
            Err(_) => return,
        };

        // Extract each known field
        self.enable = mapping.get("enable").and_then(bool_val).unwrap_or(true);
        self.show_icon = mapping.get("show_icon").and_then(bool_val).unwrap_or(true);
        self.show_notifications = mapping.get("show_notifications").and_then(bool_val).unwrap_or(true);
        self.auto_restart = mapping.get("auto_restart").and_then(bool_val).unwrap_or(true);
        if let Some(s) = mapping.get("stats") {
            if let serde_norway::Value::Mapping(ref m) = s {
                self.stats_enabled = m.get("enabled").and_then(bool_val).unwrap_or(false);
            }
        }
        self.backend = mapping.get("backend").and_then(str_val).unwrap_or("auto").to_string();
        self.clipboard_threshold = mapping.get("clipboard_threshold").and_then(int_val).unwrap_or(100).to_string();
        self.inject_delay = mapping.get("inject_delay").and_then(int_val).unwrap_or(0).to_string();
        self.key_delay = mapping.get("key_delay").and_then(int_val).unwrap_or(5).to_string();
        self.pre_paste_delay = mapping.get("pre_paste_delay").and_then(int_val).unwrap_or(100).to_string();
        self.paste_shortcut = mapping.get("paste_shortcut").and_then(str_val).unwrap_or("").to_string();
        self.paste_shortcut_event_delay = mapping.get("paste_shortcut_event_delay").and_then(int_val).unwrap_or(10).to_string();
        self.preserve_clipboard = mapping.get("preserve_clipboard").and_then(bool_val).unwrap_or(true);
        self.restore_clipboard_delay = mapping.get("restore_clipboard_delay").and_then(int_val).unwrap_or(300).to_string();
        self.toggle_key = mapping.get("toggle_key").and_then(str_val).unwrap_or("").to_string();
        self.search_trigger = mapping.get("search_trigger").and_then(str_val).unwrap_or("").to_string();
        self.search_shortcut = mapping.get("search_shortcut").and_then(str_val).unwrap_or("ALT+SPACE").to_string();
        self.word_separators = mapping.get("word_separators").and_then(str_val).unwrap_or("").to_string();
        self.backspace_limit = mapping.get("backspace_limit").and_then(int_val).unwrap_or(5).to_string();
        self.undo_backspace = mapping.get("undo_backspace").and_then(bool_val).unwrap_or(true);
        self.apply_patch = mapping.get("apply_patch").and_then(bool_val).unwrap_or(true);
        self.emulate_alt_codes = mapping.get("emulate_alt_codes").and_then(bool_val).unwrap_or(true);
        self.max_form_width = mapping.get("max_form_width").and_then(int_val).unwrap_or(700).to_string();
        self.max_form_height = mapping.get("max_form_height").and_then(int_val).unwrap_or(500).to_string();
    }

    /// Save settings back to default.yml.
    pub fn save_settings(&mut self) {
        let config_path = match &self.config_dir {
            Some(d) => d.join("config").join("default.yml"),
            None => {
                self.save_message = Some("No config directory configured".to_string());
                return;
            }
        };

        if let Err(e) = fs::create_dir_all(config_path.parent().unwrap()) {
            self.save_message = Some(format!("Cannot create config dir: {}", e));
            return;
        }

        let yaml = self.to_yaml();
        match fs::write(&config_path, yaml) {
            Ok(()) => self.save_message = Some("Settings saved".to_string()),
            Err(e) => self.save_message = Some(format!("Failed to save: {}", e)),
        }
    }

    /// Serialize current settings to a YAML string.
    fn to_yaml(&self) -> String {
        let mut lines = Vec::new();

        // General
        lines.push(format!("enable: {}", self.enable));
        lines.push(format!("show_icon: {}", self.show_icon));
        lines.push(format!("show_notifications: {}", self.show_notifications));
        lines.push(format!("auto_restart: {}", self.auto_restart));
        if !self.toggle_key.is_empty() {
            lines.push(format!("toggle_key: \"{}\"", self.toggle_key));
        }

        // Stats
        lines.push("stats:".to_string());
        lines.push(format!("  enabled: {}", self.stats_enabled));

        // Injection
        if self.backend != "auto" {
            lines.push(format!("backend: {}", self.backend));
        }
        if let Ok(v) = self.clipboard_threshold.parse::<u32>() {
            if v != 100 { lines.push(format!("clipboard_threshold: {}", v)); }
        }
        if let Ok(v) = self.inject_delay.parse::<u32>() {
            if v != 0 { lines.push(format!("inject_delay: {}", v)); }
        }
        if let Ok(v) = self.key_delay.parse::<u32>() {
            if v != 5 { lines.push(format!("key_delay: {}", v)); }
        }
        if let Ok(v) = self.pre_paste_delay.parse::<u32>() {
            if v != 100 { lines.push(format!("pre_paste_delay: {}", v)); }
        }
        if !self.paste_shortcut.is_empty() {
            lines.push(format!("paste_shortcut: \"{}\"", self.paste_shortcut));
        }
        if let Ok(v) = self.paste_shortcut_event_delay.parse::<u32>() {
            if v != 10 { lines.push(format!("paste_shortcut_event_delay: {}", v)); }
        }
        if !self.preserve_clipboard { lines.push("preserve_clipboard: false".to_string()); }
        if let Ok(v) = self.restore_clipboard_delay.parse::<u32>() {
            if v != 300 { lines.push(format!("restore_clipboard_delay: {}", v)); }
        }

        // Shortcuts
        if !self.search_trigger.is_empty() {
            lines.push(format!("search_trigger: \"{}\"", self.search_trigger));
        }
        if self.search_shortcut != "ALT+SPACE" {
            lines.push(format!("search_shortcut: \"{}\"", self.search_shortcut));
        }

        // Advanced
        if !self.word_separators.is_empty() {
            lines.push(format!("word_separators: \"{}\"", self.word_separators));
        }
        if let Ok(v) = self.backspace_limit.parse::<u32>() {
            if v != 5 { lines.push(format!("backspace_limit: {}", v)); }
        }
        if !self.undo_backspace { lines.push("undo_backspace: false".to_string()); }
        if !self.apply_patch { lines.push("apply_patch: false".to_string()); }
        if !self.emulate_alt_codes { lines.push("emulate_alt_codes: false".to_string()); }
        if let Ok(v) = self.max_form_width.parse::<u32>() {
            if v != 700 { lines.push(format!("max_form_width: {}", v)); }
        }
        if let Ok(v) = self.max_form_height.parse::<u32>() {
            if v != 500 { lines.push(format!("max_form_height: {}", v)); }
        }

        lines.join("\n") + "\n"
    }
}

// === YAML value extractors ===

fn bool_val(v: &serde_norway::Value) -> Option<bool> {
    match v {
        serde_norway::Value::Bool(b) => Some(*b),
        serde_norway::Value::String(s) => match s.to_lowercase().as_str() {
            "true" | "yes" => Some(true),
            "false" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn str_val(v: &serde_norway::Value) -> Option<&str> {
    match v {
        serde_norway::Value::String(s) => Some(s.as_str()),
        _ => None,
    }
}

fn int_val(v: &serde_norway::Value) -> Option<u32> {
    match v {
        serde_norway::Value::Number(n) => {
            if let Some(i) = n.as_u64() {
                Some(i as u32)
            } else if let Some(i) = n.as_i64() {
                Some(i as u32)
            } else if let Some(f) = n.as_f64() {
                Some(f as u32)
            } else {
                None
            }
        }
        serde_norway::Value::String(s) => s.parse::<u32>().ok(),
        _ => None,
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
                    setting_dropdown(ui, &mut state.backend, st.map_or("Backend", |s| s.backend.as_str()), &["auto", "inject", "clipboard"], st.map_or("", |s| s.backend_desc.as_str()));
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

            // Save / Reset buttons
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

// === Setting widget helpers ===

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

fn setting_dropdown(
    ui: &mut egui::Ui,
    value: &mut String,
    label: &str,
    options: &[&str],
    desc: &str,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(format!("set_{}", label))
            .selected_text(value.clone())
            .show_ui(ui, |ui| {
                for opt in options {
                    if ui.selectable_label(&value == opt, *opt).clicked() {
                        *value = opt.to_string();
                    }
                }
            });
    });
    if !desc.is_empty() {
        ui.small(egui::RichText::new(desc).color(ui.visuals().weak_text_color()));
    }
}

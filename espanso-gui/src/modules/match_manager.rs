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

//! Module 1: Match Manager — browse, create, edit, and delete matches.

use std::path::PathBuf;

use crate::backend::match_store::{MatchCache, MatchFilter, MatchType};
use crate::i18n::Translations;
use crate::widgets::confirm_dialog;

// Types formerly in widgets::match_editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectMode {
    Auto,
    Keys,
    Clipboard,
}

#[derive(Debug, Clone)]
pub struct MatchEditorState {
    pub match_type: MatchType,
    pub trigger: String,
    pub replace: String,
    pub is_word: bool,
    pub propagate_case: bool,
    pub inject_mode: InjectMode,
    pub label: String,
    pub search_terms: String,
    pub image_path: String,
    pub use_regex: bool,
}

impl Default for MatchEditorState {
    fn default() -> Self {
        MatchEditorState {
            match_type: MatchType::Text,
            trigger: String::new(),
            replace: String::new(),
            is_word: false,
            propagate_case: false,
            inject_mode: InjectMode::Auto,
            label: String::new(),
            search_terms: String::new(),
            image_path: String::new(),
            use_regex: false,
        }
    }
}

/// State for the match manager module.
pub struct MatchManagerState {
    search_query: String,
    type_filter: Option<MatchType>,
    cache: MatchCache,
    pub show_editor: bool,
    pub editing_match_id: Option<i32>,
    editor: MatchEditorState,
    confirm_delete_id: Option<i32>,
    config_dir: Option<PathBuf>,
}

impl MatchManagerState {
    pub fn new(config_dir: Option<PathBuf>) -> Self {
        MatchManagerState {
            search_query: String::new(),
            type_filter: None,
            cache: MatchCache::new(config_dir.clone()),
            show_editor: false,
            editing_match_id: None,
            editor: MatchEditorState::default(),
            confirm_delete_id: None,
            config_dir,
        }
    }

    pub fn open_new(&mut self) {
        self.editor = MatchEditorState::default();
        self.editing_match_id = None;
        self.show_editor = true;
    }

    pub fn open_edit(&mut self, match_id: i32) {
        if let Some(m) = self.cache.get(match_id) {
            self.editor = MatchEditorState {
                match_type: m.match_type,
                trigger: m.trigger_display.clone(),
                replace: m.replace_preview.clone(),
                is_word: m.is_word,
                propagate_case: m.propagate_case,
                inject_mode: InjectMode::Auto,
                label: m.label.clone().unwrap_or_default(),
                search_terms: m.search_terms.join(", "),
                image_path: m.image_path.clone().unwrap_or_default(),
                use_regex: m.match_type == MatchType::Regex,
            };
            self.editing_match_id = Some(match_id);
            self.show_editor = true;
        }
    }

    pub fn handle_save(&mut self) -> String {
        if let Some(ref config_dir) = self.config_dir {
            let trigger = if self.editor.trigger.is_empty() {
                ":untitled".to_string()
            } else {
                self.editor.trigger.clone()
            };
            let replace = self.editor.replace.clone();

            let yaml_content = format!(
                "\n  - trigger: \"{}\"\n    replace: \"{}\"\n",
                trigger, replace
            );

            let match_dir = config_dir.join("match");
            if let Err(e) = std::fs::create_dir_all(&match_dir) {
                return format!("Failed to create match directory: {}", e);
            }

            let base_file = match_dir.join("base.yml");
            let current = std::fs::read_to_string(&base_file).unwrap_or_default();
            let new_content = current + &yaml_content;

            match std::fs::write(&base_file, &new_content) {
                Ok(()) => {
                    self.cache.mark_stale();
                    self.show_editor = false;
                    "Match saved".to_string()
                }
                Err(e) => format!("Failed to save: {}", e),
            }
        } else {
            "No config directory configured".to_string()
        }
    }

    pub fn handle_delete(&mut self, match_id: i32) {
        self.confirm_delete_id = Some(match_id);
    }

    pub fn execute_delete(&mut self) {
        if self.confirm_delete_id.take().is_some() {
            self.cache.mark_stale();
        }
    }
}

/// Render the match manager module.
pub fn show(ui: &mut egui::Ui, state: &mut MatchManagerState, t: &Translations) {
    let mm = t.match_manager.as_ref();
    let cm = t.common.as_ref();

    let mut edit_id = None;
    let mut delete_id = None;

    ui.vertical(|ui| {
        // === Header ===
        ui.horizontal(|ui| {
            ui.heading(mm.map_or("Match Manager", |m| m.title.as_str()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(mm.map_or("+ New", |m| m.add_button.as_str()))
                            .fill(egui::Color32::from_rgb(34, 139, 34)),
                    )
                    .clicked()
                {
                    state.open_new();
                }
            });
        });

        ui.add_space(8.0);

        // === Search and filter ===
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.search_query)
                    .hint_text(mm.map_or("Search matches...", |m| m.search_placeholder.as_str()))
                    .desired_width(300.0),
            );

            let filter_label = match &state.type_filter {
                Some(mt) => mt.label(),
                None => mm.map_or("All Types", |m| m.type_filter_all.as_str()),
            };
            egui::ComboBox::from_id_salt("type_filter_mm")
                .selected_text(filter_label)
                .width(120.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(state.type_filter.is_none(), mm.map_or("All Types", |m| m.type_filter_all.as_str()))
                        .clicked()
                    {
                        state.type_filter = None;
                    }
                    for mtype in &[
                        MatchType::Text,
                        MatchType::Image,
                        MatchType::Markdown,
                        MatchType::Regex,
                        MatchType::Script,
                    ] {
                        if ui
                            .selectable_label(state.type_filter.as_ref() == Some(mtype), mtype.label())
                            .clicked()
                        {
                            state.type_filter = Some(*mtype);
                        }
                    }
                });
        });

        ui.add_space(8.0);

        // === Match list ===
        let filter = MatchFilter {
            search_query: std::mem::take(&mut state.search_query),
            type_filter: state.type_filter,
        };
        let search_str = filter.search_query.clone();
        let matches = state.cache.filtered(&filter);
        state.search_query = search_str;

        if matches.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("📭").size(48.0));
                ui.add_space(8.0);
                ui.strong(mm.map_or("No matches yet", |m| m.no_matches.as_str()));
            });
        } else {
            let types_count = {
                let mut types = std::collections::HashSet::new();
                for m in &matches {
                    types.insert(m.match_type);
                }
                types.len()
            };
            let total = matches.len();

            ui.small({
                let s = mm.map_or(
                    format!("{} matches · {} types", total, types_count),
                    |m| m.total_count.clone(),
                );
                s.replace("{}", &total.to_string())
                    .replace("{}", &types_count.to_string())
            });

            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 40.0)
                .show(ui, |ui| {
                    for m in &matches {
                        let response = ui.horizontal(|ui| {
                            let icon = match m.match_type {
                                MatchType::Image => "🖼",
                                MatchType::Text => "📝",
                                MatchType::Markdown => "📋",
                                MatchType::Html => "🌐",
                                MatchType::Regex => "🔍",
                                MatchType::Script => "⚡",
                            };
                            ui.label(icon);
                            ui.vertical(|ui| {
                                ui.strong(&m.trigger_display);
                                ui.small(egui::RichText::new(format!("→ {}", m.replace_preview))
                                    .color(ui.visuals().weak_text_color()));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✏️").clicked() {
                                    edit_id = Some(m.id);
                                }
                                if ui.small_button("🗑").clicked() {
                                    delete_id = Some(m.id);
                                }
                                ui.label(m.match_type.label());
                            });
                        });

                        let r = response.response;
                        if r.hovered() {
                            ui.painter().rect_filled(
                                r.rect,
                                4.0,
                                if ui.visuals().dark_mode {
                                    egui::Color32::from_white_alpha(10)
                                } else {
                                    egui::Color32::from_black_alpha(5)
                                },
                            );
                        }
                    }
                });
        }
    });

    if let Some(id) = edit_id {
        state.open_edit(id);
    }
    if let Some(id) = delete_id {
        state.handle_delete(id);
    }

    // === Editor dialog ===
    if state.show_editor {
        let is_new = state.editing_match_id.is_none();
        let title = if is_new {
            mm.map_or("New Match".to_string(), |m| m.editor_title_new.clone())
        } else {
            mm.map_or("Edit Match".to_string(), |m| m.editor_title_edit.clone())
        };

        let mut should_save = false;
        let mut should_cancel = false;
        let mut open = true;

        egui::Window::new(title)
            .open(&mut open)
            .resizable(true)
            .default_width(500.0)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Type selector
                    ui.horizontal(|ui| {
                        ui.label(mm.map_or("Type:", |m| m.editor_type.as_str()));
                        for mtype in &[
                            MatchType::Text,
                            MatchType::Image,
                            MatchType::Markdown,
                            MatchType::Regex,
                            MatchType::Script,
                        ] {
                            let selected = state.editor.match_type == *mtype;
                            if ui.selectable_label(selected, mtype.label()).clicked() {
                                state.editor.match_type = *mtype;
                            }
                        }
                    });

                    ui.add_space(8.0);

                    // Trigger
                    ui.horizontal(|ui| {
                        ui.label(mm.map_or("Trigger:", |m| m.editor_trigger.as_str()));
                        ui.add(
                            egui::TextEdit::singleline(&mut state.editor.trigger)
                                .hint_text(if state.editor.use_regex {
                                    "/regex/"
                                } else {
                                    ":hello"
                                })
                                .desired_width(300.0),
                        );
                    });

                    ui.add_space(8.0);

                    // Replace content
                    if state.editor.match_type == MatchType::Image {
                        ui.horizontal(|ui| {
                            ui.label("Image:");
                            ui.add(
                                egui::TextEdit::singleline(&mut state.editor.image_path)
                                    .hint_text("/path/to/image.png")
                                    .desired_width(300.0),
                            );
                        });
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp", "svg", "webp"])
                                .pick_file()
                            {
                                state.editor.image_path = path.to_string_lossy().to_string();
                            }
                        }
                    } else {
                        ui.label(mm.map_or("Replacement:", |m| m.editor_replace.as_str()));
                        ui.add(
                            egui::TextEdit::multiline(&mut state.editor.replace)
                                .hint_text("Hi There!")
                                .desired_width(f32::INFINITY)
                                .desired_rows(3),
                        );
                    }

                    ui.add_space(8.0);

                    // Advanced options
                    egui::CollapsingHeader::new(
                        mm.map_or("Advanced Options", |m| m.editor_advanced.as_str()),
                    )
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.checkbox(
                            &mut state.editor.is_word,
                            mm.map_or("Word boundary", |m| m.editor_word_boundary.as_str()),
                        );
                        ui.checkbox(
                            &mut state.editor.propagate_case,
                            mm.map_or("Propagate case", |m| m.editor_propagate_case.as_str()),
                        );
                    });

                    ui.add_space(16.0);

                    // Buttons
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(mm.map_or("Cancel", |m| m.editor_cancel.as_str()))
                                .clicked()
                            {
                                should_cancel = true;
                            }
                            if ui
                                .add(
                                    egui::Button::new(mm.map_or("Save", |m| m.editor_save.as_str()))
                                        .fill(egui::Color32::from_rgb(34, 139, 34)),
                                )
                                .clicked()
                            {
                                should_save = true;
                            }
                        });
                    });
                });
            });

        if !open {
            should_cancel = true;
        }
        if should_cancel {
            state.show_editor = false;
        }
        if should_save {
            let _msg = state.handle_save();
        }
    }

    // === Delete confirmation ===
    if state.confirm_delete_id.is_some() {
        let result = confirm_dialog::show_confirm(
            ui.ctx(),
            mm.map_or("Delete Match", |m| m.confirm_delete.as_str()),
            mm.map_or("Delete this match?", |m| m.confirm_delete.as_str()),
            Some(mm.map_or(
                "Deleted matches can be recovered from trash.",
                |m| m.confirm_delete_hint.as_str(),
            )),
            cm.map_or("Delete", |c| c.delete.as_str()),
            cm.map_or("Cancel", |c| c.cancel.as_str()),
            true,
        );

        if let Some(true) = result {
            state.execute_delete();
        } else if result.is_some() {
            state.confirm_delete_id = None;
        }
    }
}

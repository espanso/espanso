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

use std::fs;
use std::path::PathBuf;

use crate::backend::match_store::{MatchCache, MatchFilter, MatchType};
use crate::i18n::Translations;
use crate::widgets::confirm_dialog;
use crate::widgets::thumbnail;

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
    confirm_delete_trigger: Option<String>,
    config_dir: Option<PathBuf>,
    status_message: Option<String>,
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
            confirm_delete_trigger: None,
            config_dir,
            status_message: None,
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

    /// Save a match to the YAML file.
    /// For new matches: append to base.yml.
    /// For edits: rewrite the match entry in place (simplified: remove old + append new).
    pub fn handle_save(&mut self) {
        let config_dir = match &self.config_dir {
            Some(d) => d.clone(),
            None => {
                let msg = "No config directory configured".to_string();
                self.status_message = Some(msg);
                return;
            }
        };

        let trigger = if self.editor.trigger.is_empty() {
            ":untitled".to_string()
        } else {
            // Ensure trigger starts with meaningful content
            let t = self.editor.trigger.trim().to_string();
            if t.is_empty() { ":untitled".to_string() } else { t }
        };

        let replace = self.editor.replace.clone();

        let match_dir = config_dir.join("match");
        if let Err(e) = fs::create_dir_all(&match_dir) {
            let msg = format!("Failed to create match directory: {}", e);
            self.status_message = Some(msg);
            return;
        }

        let base_file = match_dir.join("base.yml");

        // Read current content
        let mut current = fs::read_to_string(&base_file).unwrap_or_else(|_| "matches:\n".to_string());

        // If editing an existing match, remove the old entry first
        if let Some(edit_id) = self.editing_match_id {
            if let Some(old_match) = self.cache.get(edit_id) {
                current = remove_match_from_yaml(&current, &old_match.trigger_display);
            }
        }

        // Append the new match as a properly formatted YAML entry
        let escaped_trigger = yaml_escape(&trigger);
        let escaped_replace = yaml_escape(&replace);
        let yaml_fragment = format!(
            "  - trigger: \"{}\"\n    replace: \"{}\"\n",
            escaped_trigger, escaped_replace
        );

        // Ensure the file ends with a newline before appending
        if !current.ends_with('\n') {
            current.push('\n');
        }
        current.push_str(&yaml_fragment);

        match fs::write(&base_file, &current) {
            Ok(()) => {
                self.cache.mark_stale();
                self.show_editor = false;
                self.status_message = Some("Match saved".to_string());
            }
            Err(e) => {
                let msg = format!("Failed to save: {}", e);
                self.status_message = Some(msg);
            }
        }
    }

    pub fn handle_delete(&mut self, match_id: i32) {
        self.confirm_delete_id = Some(match_id);
        if let Some(m) = self.cache.get(match_id) {
            self.confirm_delete_trigger = Some(m.trigger_display.clone());
        }
    }

    /// Actually remove the match from the YAML file.
    pub fn execute_delete(&mut self) {
        let trigger = match self.confirm_delete_trigger.take() {
            Some(t) => t,
            None => return,
        };
        self.confirm_delete_id = None;

        let config_dir = match &self.config_dir {
            Some(d) => d.clone(),
            None => {
                self.status_message = Some("No config directory configured".to_string());
                return;
            }
        };

        let base_file = config_dir.join("match").join("base.yml");

        // Read current content, remove the match entry, write back
        match fs::read_to_string(&base_file) {
            Ok(current) => {
                let updated = remove_match_from_yaml(&current, &trigger);
                match fs::write(&base_file, &updated) {
                    Ok(()) => {
                        self.cache.mark_stale();
                        self.status_message = Some("Match deleted".to_string());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Failed to delete: {}", e));
                    }
                }
            }
            Err(e) => {
                self.status_message = Some(format!("Cannot read file: {}", e));
            }
        }
    }
}

// === YAML helpers ===

/// Escape a string for safe YAML double-quoted insertion.
fn yaml_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

/// Remove a match entry with the given trigger from a YAML content string.
/// Scans for `- trigger: "THE_TRIGGER"` and removes that entire match block
/// (the header line plus all subsequent indented property lines).
fn remove_match_from_yaml(content: &str, trigger: &str) -> String {
    let target = format!("- trigger: \"{}\"", trigger);
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<&str> = Vec::new();
    let mut skipping = false;

    for line in &lines {
        // An indented match property starts with at least 2 spaces.
        // Match headers start with "  - ", and their properties start with "    ".
        let is_indented = line.starts_with("  ");

        if !skipping {
            // Check if this line is the start of the match to remove
            if line.trim() == target {
                skipping = true;
                continue;
            }
            result.push(line);
        } else {
            // We are inside the match block being removed.
            // Still inside if the line is indented (but not the start of a new match).
            if is_indented && !line.trim().starts_with("- ") {
                // Property line of the match being removed — skip it.
                continue;
            }
            // Otherwise we have reached the boundary of the block.
            // The boundary can be: a new match header ("  - trigger:" / "  - regex:"),
            // a top-level key (non-empty, non-indented), or end of file.
            skipping = false;
            // Re-process this line in the non-skipping path so it is kept.
            if line.trim() == target {
                skipping = true;
                continue;
            }
            result.push(line);
        }
    }

    // Remove consecutive empty lines (max 1 blank line)
    let mut cleaned = Vec::new();
    let mut prev_empty = false;
    for &line in &result {
        let is_empty = line.trim().is_empty();
        if is_empty && prev_empty {
            continue;
        }
        prev_empty = is_empty;
        cleaned.push(line);
    }

    cleaned.join("\n")
}

fn match_type_badge_color(m: &crate::backend::match_store::GuiMatch) -> egui::Color32 {
    match m.match_type {
        MatchType::Text => egui::Color32::from_rgb(72, 199, 142),
        MatchType::Markdown => egui::Color32::from_rgb(94, 145, 255),
        MatchType::Html => egui::Color32::from_rgb(255, 140, 0),
        MatchType::Regex => egui::Color32::from_rgb(180, 100, 220),
        MatchType::Script => egui::Color32::from_rgb(255, 200, 40),
        MatchType::Image => egui::Color32::from_rgb(100, 149, 237),
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

        // === Status message ===
        if let Some(ref msg) = state.status_message.clone() {
            if msg.contains("Failed") || msg.contains("Cannot") {
                ui.colored_label(egui::Color32::from_rgb(255, 107, 107), msg);
            } else {
                ui.colored_label(egui::Color32::from_rgb(72, 199, 142), msg);
            }
        }

        ui.add_space(4.0);

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
                        let selected = false;

                        let base_response = egui::Frame::none()
                            .fill(if selected {
                                ui.visuals().selection.bg_fill
                            } else {
                                egui::Color32::TRANSPARENT
                            })
                            .rounding(6.0)
                            .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                            .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // === Preview thumbnail / icon column ===
                                    if m.is_image {
                                        let img_path = m.image_path.as_deref();
                                        let cfg_dir = state.config_dir.as_ref();
                                        thumbnail::show_thumbnail(
                                            ui,
                                            img_path,
                                            cfg_dir,
                                            egui::vec2(64.0, 48.0),
                                        );
                                    } else {
                                        // Show a styled preview card for non-image matches
                                        let preview_size = egui::vec2(64.0, 48.0);
                                        let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
                                        let bg = if ui.visuals().dark_mode {
                                            egui::Color32::from_rgb(30, 35, 50)
                                        } else {
                                            egui::Color32::from_rgb(240, 245, 250)
                                        };
                                        ui.painter().rect_filled(rect, 4.0, bg);

                                        // Type badge inside preview card
                                        let icon = match m.match_type {
                                            MatchType::Text => "Aa",
                                            MatchType::Markdown => "MD",
                                            MatchType::Html => "<>",
                                            MatchType::Regex => ".*",
                                            MatchType::Script => "⚡",
                                            MatchType::Image => "🖼",
                                        };
                                        let color = match m.match_type {
                                            MatchType::Text => egui::Color32::from_rgb(72, 199, 142),
                                            MatchType::Markdown => egui::Color32::from_rgb(94, 145, 255),
                                            MatchType::Html => egui::Color32::from_rgb(255, 140, 0),
                                            MatchType::Regex => egui::Color32::from_rgb(180, 100, 220),
                                            MatchType::Script => egui::Color32::from_rgb(255, 200, 40),
                                            MatchType::Image => egui::Color32::from_rgb(100, 149, 237),
                                        };
                                        ui.painter().text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            icon,
                                            egui::FontId::proportional(14.0),
                                            color,
                                        );

                                        // Show snippet of replacement text in preview card
                                        let preview = if m.replace_preview.len() > 20 {
                                            format!("{}...", &m.replace_preview[..17])
                                        } else {
                                            m.replace_preview.clone()
                                        };
                                        if !preview.is_empty() && m.match_type != MatchType::Script {
                                            ui.painter().text(
                                                egui::pos2(rect.center().x, rect.bottom() - 10.0),
                                                egui::Align2::CENTER_BOTTOM,
                                                preview,
                                                egui::FontId::proportional(8.0),
                                                ui.visuals().weak_text_color(),
                                            );
                                        }
                                    }

                                    ui.add_space(8.0);

                                    // === Main info column ===
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.strong(&m.trigger_display);

                                            // Label badge
                                            if let Some(ref label) = m.label {
                                                if !label.is_empty() {
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        egui::RichText::new(label)
                                                            .size(10.0)
                                                            .background_color(if ui.visuals().dark_mode {
                                                                egui::Color32::from_gray(60)
                                                            } else {
                                                                egui::Color32::from_gray(220)
                                                            }),
                                                    );
                                                }
                                            }
                                        });

                                        // Rich preview of replacement content
                                        let preview_text = if m.is_image {
                                            let path = m.image_path.as_deref().unwrap_or(m.replace_preview.as_str());
                                            format!("🖼 Image: {}", path)
                                        } else if m.match_type == MatchType::Script {
                                            format!("⚡ Script: {}", m.replace_preview)
                                        } else if m.match_type == MatchType::Markdown {
                                            format!("📋 Markdown: {}", m.replace_preview)
                                        } else {
                                            format!("→ {}", m.replace_preview)
                                        };

                                        ui.small(
                                            egui::RichText::new(preview_text)
                                                .color(ui.visuals().weak_text_color()),
                                        );

                                        // Extra metadata row
                                        ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 4.0;
                                            let type_badge = match_type_badge_color(m);
                                            ui.label(
                                                egui::RichText::new(m.match_type.label())
                                                    .size(9.0)
                                                    .color(type_badge),
                                            );
                                            if m.is_word {
                                                ui.label(
                                                    egui::RichText::new("Word")
                                                        .size(9.0)
                                                        .color(ui.visuals().weak_text_color()),
                                                );
                                            }
                                            if m.propagate_case {
                                                ui.label(
                                                    egui::RichText::new("Case")
                                                        .size(9.0)
                                                        .color(ui.visuals().weak_text_color()),
                                                );
                                            }
                                            // Source file
                                            ui.label(
                                                egui::RichText::new(format!("● {}", m.source_file))
                                                    .size(9.0)
                                                    .color(ui.visuals().weak_text_color()),
                                            );
                                        });
                                    });

                                    // === Action buttons ===
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("✏️").clicked() {
                                            edit_id = Some(m.id);
                                        }
                                        if ui.small_button("🗑").clicked() {
                                            delete_id = Some(m.id);
                                        }
                                    });
                                });
                            });

                        let r = base_response.response;
                        if r.hovered() {
                            ui.painter().rect_filled(
                                r.rect,
                                6.0,
                                if ui.visuals().dark_mode {
                                    egui::Color32::from_white_alpha(8)
                                } else {
                                    egui::Color32::from_black_alpha(4)
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
            state.editing_match_id = None;
        }
        if should_save {
            state.handle_save();
        }
    }

    // === Delete confirmation ===
    if state.confirm_delete_id.is_some() {
        let trigger = state.confirm_delete_trigger.clone().unwrap_or_default();
        let result = confirm_dialog::show_confirm(
            ui.ctx(),
            mm.map_or("Delete Match", |m| m.confirm_delete.as_str()),
            &format!("Delete \"{}\"?", trigger),
            Some(mm.map_or(
                "The match will be removed from the YAML file.",
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
            state.confirm_delete_trigger = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_match_from_yaml_first() {
        let input = "matches:\n  - trigger: \":hello\"\n    replace: \"Hi!\"\n  - trigger: \":bye\"\n    replace: \"Bye!\"\n";
        let result = remove_match_from_yaml(input, ":hello");
        assert_eq!(result, "matches:\n  - trigger: \":bye\"\n    replace: \"Bye!\"");
    }

    #[test]
    fn test_remove_match_from_yaml_last() {
        let input = "matches:\n  - trigger: \":hello\"\n    replace: \"Hi!\"\n  - trigger: \":bye\"\n    replace: \"Bye!\"\n";
        let result = remove_match_from_yaml(input, ":bye");
        assert_eq!(result, "matches:\n  - trigger: \":hello\"\n    replace: \"Hi!\"");
    }

    #[test]
    fn test_remove_match_from_yaml_with_extra_fields() {
        let input = "matches:\n  - trigger: \":foo\"\n    replace: \"Foo!\"\n    is_word: true\n    propagate_case: false\n  - trigger: \":bar\"\n    replace: \"Bar!\"\n";
        let result = remove_match_from_yaml(input, ":foo");
        assert_eq!(result, "matches:\n  - trigger: \":bar\"\n    replace: \"Bar!\"");
    }

    #[test]
    fn test_remove_match_from_yaml_only_entry() {
        let input = "matches:\n  - trigger: \":hello\"\n    replace: \"Hi!\"\n";
        let result = remove_match_from_yaml(input, ":hello");
        assert_eq!(result, "matches:");
    }

    #[test]
    fn test_remove_match_from_yaml_no_orphaned_lines() {
        // Regression test: the delete must not leave indented property lines behind.
        let input = "matches:\n  - trigger: \":hello\"\n    replace: \"Hi!\"\n  - trigger: \":bye\"\n    replace: \"Bye!\"\n";
        let result = remove_match_from_yaml(input, ":hello");
        // The orphaned "    replace: \"Hi!\"" must NOT appear in output
        assert!(!result.contains("Hi!"), "orphaned replace line leaked");
    }

    #[test]
    fn test_yaml_escape() {
        assert_eq!(yaml_escape("hello"), "hello");
        assert_eq!(yaml_escape("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(yaml_escape("path\\to"), "path\\\\to");
    }
}

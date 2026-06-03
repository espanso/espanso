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

use egui::Color32;

/// A reusable search bar widget.
pub struct SearchBar {
    text: String,
    hint: String,
}

impl SearchBar {
    pub fn new(hint: impl Into<String>) -> Self {
        SearchBar {
            text: String::new(),
            hint: hint.into(),
        }
    }

    /// Show the search bar and return the current text.
    pub fn show(&mut self, ui: &mut egui::Ui) -> &str {
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.text)
                .hint_text(&self.hint)
                .desired_width(f32::INFINITY),
        );

        if self.text.is_empty() {
            // Show search icon when empty
            let icon_pos = response.rect.right_center() - egui::vec2(24.0, 0.0);
            ui.painter().text(
                icon_pos,
                egui::Align2::RIGHT_CENTER,
                "🔍",
                egui::FontId::proportional(14.0),
                if ui.visuals().dark_mode {
                    Color32::from_gray(150)
                } else {
                    Color32::from_gray(180)
                },
            );
        } else {
            // Show clear button when has text
            let clear_pos = response.rect.right_center() - egui::vec2(20.0, 0.0);
            if ui
                .put(
                    egui::Rect::from_center_size(clear_pos, egui::vec2(20.0, 20.0)),
                    egui::Button::new("✕"),
                )
                .clicked()
            {
                self.clear();
            }
        }

        &self.text
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }
}

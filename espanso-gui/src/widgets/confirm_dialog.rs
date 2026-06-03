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

/// Show a confirmation dialog and return `true` if confirmed.
///
/// Returns `None` if the dialog is still open (no choice made yet).
pub fn show_confirm(
    ctx: &egui::Context,
    title: &str,
    message: &str,
    hint: Option<&str>,
    confirm_label: &str,
    cancel_label: &str,
    is_dangerous: bool,
) -> Option<bool> {
    let mut result = None;
    let mut open = true;

    egui::Window::new(title)
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_min_width(300.0);
            ui.label(message);

            if let Some(hint_text) = hint {
                ui.add_space(4.0);
                ui.small(
                    egui::RichText::new(hint_text)
                        .color(ui.visuals().weak_text_color()),
                );
            }

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(cancel_label).clicked() {
                        result = Some(false);
                    }
                    let confirm_btn = if is_dangerous {
                        egui::Button::new(confirm_label)
                            .fill(egui::Color32::from_rgb(220, 53, 69))
                    } else {
                        egui::Button::new(confirm_label)
                            .fill(egui::Color32::from_rgb(34, 139, 34))
                    };
                    if ui.add(confirm_btn).clicked() {
                        result = Some(true);
                    }
                });
            });
        });

    if !open {
        result = Some(false);
    }

    result
}

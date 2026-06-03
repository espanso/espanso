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

use crate::backend::match_store::GuiMatch;

/// Render a single match row in the match list.
pub fn match_row(
    ui: &mut egui::Ui,
    m: &GuiMatch,
    selected: bool,
    on_edit: impl FnOnce(),
    on_delete: impl FnOnce(),
) -> egui::Response {
    let icon = match_type_icon(m);
    let type_badge = match_type_badge(m);

    let base_response = egui::Frame::none()
        .fill(if selected {
            ui.visuals().selection.bg_fill
        } else {
            egui::Color32::TRANSPARENT
        })
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Icon
                ui.label(icon);

                // Trigger + replace preview
                ui.vertical(|ui| {
                    ui.strong(&m.trigger_display);
                    ui.small(
                        egui::RichText::new(format!("→ {}", m.replace_preview))
                            .color(ui.visuals().weak_text_color()),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Edit button
                    if ui.small_button("✏️").clicked() {
                        on_edit();
                    }
                    // Delete button
                    if ui.small_button("🗑").clicked() {
                        on_delete();
                    }
                });

                // Type badge
                ui.label(type_badge);
            });
        });

    let response = base_response.response;

    // Hover effect
    if response.hovered() && !selected {
        ui.painter().rect_filled(
            response.rect,
            4.0,
            if ui.visuals().dark_mode {
                egui::Color32::from_white_alpha(10)
            } else {
                egui::Color32::from_black_alpha(5)
            },
        );
    }

    response
}

fn match_type_icon(m: &GuiMatch) -> &'static str {
    if m.is_image {
        "🖼"
    } else {
        match m.match_type {
            crate::backend::match_store::MatchType::Text => "📝",
            crate::backend::match_store::MatchType::Markdown => "📋",
            crate::backend::match_store::MatchType::Html => "🌐",
            crate::backend::match_store::MatchType::Regex => "🔍",
            crate::backend::match_store::MatchType::Script => "⚡",
            crate::backend::match_store::MatchType::Image => "🖼",
        }
    }
}

fn match_type_badge(m: &GuiMatch) -> egui::RichText {
    let (label, color) = if m.is_image {
        ("IMG", egui::Color32::from_rgb(100, 149, 237))
    } else {
        match m.match_type {
            crate::backend::match_store::MatchType::Text => {
                ("TXT", egui::Color32::from_rgb(100, 200, 100))
            }
            crate::backend::match_store::MatchType::Markdown => {
                ("MD", egui::Color32::from_rgb(100, 149, 237))
            }
            crate::backend::match_store::MatchType::Html => {
                ("HTML", egui::Color32::from_rgb(255, 140, 0))
            }
            crate::backend::match_store::MatchType::Regex => {
                ("REGEX", egui::Color32::from_rgb(200, 100, 200))
            }
            crate::backend::match_store::MatchType::Script => {
                ("SCRIPT", egui::Color32::from_rgb(255, 200, 50))
            }
            crate::backend::match_store::MatchType::Image => {
                ("IMG", egui::Color32::from_rgb(100, 149, 237))
            }
        }
    };

    egui::RichText::new(label)
        .color(color)
        .size(10.0)
        .strong()
}

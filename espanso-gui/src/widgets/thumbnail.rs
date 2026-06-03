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

use std::path::PathBuf;

/// Show a thumbnail preview for an image match.
/// Falls back to an emoji placeholder if the image can't be loaded.
pub fn show_thumbnail(
    ui: &mut egui::Ui,
    image_path: Option<&str>,
    _config_dir: Option<&PathBuf>,
    size: egui::Vec2,
) {
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        // For now, show a placeholder. Full implementation will:
        // 1. Check image cache at .espanso/cache/thumbnails/
        // 2. If cache miss, load and resize the image using the `image` crate
        // 3. Convert to egui texture and display
        let color = if ui.visuals().dark_mode {
            egui::Color32::from_gray(50)
        } else {
            egui::Color32::from_gray(230)
        };

        ui.painter().rect_filled(rect, 4.0, color);

        // Placeholder icon
        let center = rect.center();
        ui.painter().text(
            center,
            egui::Align2::CENTER_CENTER,
            match image_path {
                Some(_) => "🖼️",
                None => "❓",
            },
            egui::FontId::proportional(size.y * 0.4),
            ui.visuals().widgets.inactive.text_color(),
        );

        // Show filename if available
        if let Some(path) = image_path {
            let filename = std::path::Path::new(path)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string());

            ui.painter().text(
                egui::pos2(center.x, rect.bottom() - 14.0),
                egui::Align2::CENTER_BOTTOM,
                filename,
                egui::FontId::proportional(10.0),
                ui.visuals().weak_text_color(),
            );
        }
    }
}

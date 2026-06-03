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

/// Theme mode for the GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeMode {
    /// Follow the operating system preference.
    System,
    /// Always use light theme.
    Light,
    /// Always use dark theme.
    Dark,
}

impl ThemeMode {
    pub fn label(&self) -> &'static str {
        match self {
            ThemeMode::System => "Follow System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }

    pub fn all() -> [ThemeMode; 3] {
        [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark]
    }
}

/// Detect whether the system is in dark mode.
pub fn system_is_dark() -> bool {
    match dark_light::detect() {
        dark_light::Mode::Dark => true,
        dark_light::Mode::Light => false,
        // Default to light if detection fails
        dark_light::Mode::Default => false,
    }
}

/// Apply the given theme to the egui context.
pub fn apply_theme(ctx: &egui::Context, mode: ThemeMode) {
    let is_dark = match mode {
        ThemeMode::System => system_is_dark(),
        ThemeMode::Light => false,
        ThemeMode::Dark => true,
    };

    if is_dark {
        ctx.set_visuals(egui::Visuals::dark());
    } else {
        ctx.set_visuals(egui::Visuals::light());
    }

    // Tweak some style settings for better readability
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    ctx.set_style(style);
}

/// Apply the system theme (used on first launch).
pub fn apply_system_theme(ctx: &egui::Context) {
    apply_theme(ctx, ThemeMode::System);
}

/// Accent colors used throughout the GUI.
pub struct AccentColors {
    pub primary: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub info: Color32,
}

impl AccentColors {
    pub fn for_theme(is_dark: bool) -> Self {
        if is_dark {
            AccentColors {
                primary: Color32::from_rgb(94, 145, 255),
                success: Color32::from_rgb(72, 199, 142),
                warning: Color32::from_rgb(255, 183, 77),
                danger: Color32::from_rgb(255, 107, 107),
                info: Color32::from_rgb(129, 178, 255),
            }
        } else {
            AccentColors {
                primary: Color32::from_rgb(51, 102, 255),
                success: Color32::from_rgb(34, 139, 34),
                warning: Color32::from_rgb(255, 140, 0),
                danger: Color32::from_rgb(220, 53, 69),
                info: Color32::from_rgb(13, 110, 253),
            }
        }
    }
}

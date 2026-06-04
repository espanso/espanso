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

use egui::{
    Color32, CursorIcon, Margin, Rounding, Shadow, Stroke, Style, TextCursorStyle, Vec2, Visuals,
    WidgetVisuals,
};

/// Theme mode for the GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeMode {
    System,
    Light,
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
        dark_light::Mode::Default => false,
    }
}

/// Apply the given theme to the egui context with polished styling.
pub fn apply_theme(ctx: &egui::Context, mode: ThemeMode) {
    let is_dark = match mode {
        ThemeMode::System => system_is_dark(),
        ThemeMode::Light => false,
        ThemeMode::Dark => true,
    };

    let mut style = (*ctx.style()).clone();

    if is_dark {
        apply_dark_visuals(&mut style);
    } else {
        apply_light_visuals(&mut style);
    }

    // ── Shared spacing ──────────────────────────────────────────────────────
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(16.0, 8.0);
    style.spacing.indent = 20.0;
    style.spacing.window_margin = Margin::same(8.0);
    style.spacing.menu_margin = Margin::symmetric(8.0, 4.0);
    style.spacing.interact_size = Vec2::new(48.0, 24.0);
    style.spacing.combo_width = 160.0;
    style.spacing.text_edit_width = 320.0;
    style.spacing.icon_width = 16.0;
    style.spacing.icon_width_inner = 10.0;
    style.spacing.icon_spacing = 6.0;

    // Button animation
    style.animation_time = 0.15;

    ctx.set_style(style);
}

/// Apply the system theme (used on first launch).
pub fn apply_system_theme(ctx: &egui::Context) {
    apply_theme(ctx, ThemeMode::System);
}

// ── Light theme ──────────────────────────────────────────────────────────────

fn apply_light_visuals(style: &mut Style) {
    let v = &mut style.visuals;
    *v = Visuals::light();

    let primary = Color32::from_rgb(67, 97, 238); // #4361EE
    let primary_hover = Color32::from_rgb(56, 86, 222); // #3856DE
    let primary_active = Color32::from_rgb(48, 70, 184); // #3046B8
    let text_primary = Color32::from_rgb(26, 27, 47); // #1A1B2F
    let text_secondary = Color32::from_rgb(107, 114, 128); // #6B7280
    let bg_window = Color32::from_rgb(255, 255, 255); // #FFFFFF
    let bg_panel = Color32::from_rgb(245, 246, 250); // #F5F6FA
    let bg_faint = Color32::from_rgb(241, 243, 247); // #F1F3F7
    let bg_extreme = Color32::from_rgb(249, 250, 251); // #F9FAFB
    let border = Color32::from_rgb(226, 232, 240); // #E2E8F0
    let border_strong = Color32::from_rgb(203, 213, 225); // #CBD5E1

    v.dark_mode = false;

    // ── Text & hyperlinks ───────────────────────────────────────────────────
    v.override_text_color = Some(text_primary);
    v.hyperlink_color = primary;
    v.warn_fg_color = Color32::from_rgb(245, 158, 11); // #F59E0B  amber
    v.error_fg_color = Color32::from_rgb(239, 68, 68); // #EF4444  red
    v.code_bg_color = Color32::from_rgb(233, 236, 239);
    v.faint_bg_color = bg_faint;
    v.extreme_bg_color = bg_extreme;

    // ── Selection ────────────────────────────────────────────────────────────
    v.selection.bg_fill = primary;
    v.selection.stroke = Stroke::new(1.0, primary);

    // ── Window ───────────────────────────────────────────────────────────────
    v.window_rounding = Rounding::same(12.0);
    v.window_shadow = Shadow {
        offset: Vec2::new(0.0, 4.0),
        blur: 24.0,
        spread: 0.0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 25),
    };
    v.window_fill = bg_window;
    v.window_stroke = Stroke::new(1.0, border);
    v.window_highlight_topmost = true;

    // ── Menu ─────────────────────────────────────────────────────────────────
    v.menu_rounding = Rounding::same(8.0);
    v.popup_shadow = Shadow {
        offset: Vec2::new(0.0, 10.0),
        blur: 25.0,
        spread: 0.0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 20),
    };

    // ── Panel fill ───────────────────────────────────────────────────────────
    v.panel_fill = bg_panel;

    // ── Widgets: non-interactive ─────────────────────────────────────────────
    v.widgets.noninteractive = WidgetVisuals {
        weak_bg_fill: bg_panel,
        bg_fill: bg_panel,
        bg_stroke: Stroke::new(1.0, border),
        fg_stroke: Stroke::new(1.0, text_secondary),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };

    // ── Widgets: inactive ────────────────────────────────────────────────────
    v.widgets.inactive = WidgetVisuals {
        weak_bg_fill: bg_window,
        bg_fill: Color32::from_rgb(226, 232, 240),
        bg_stroke: Stroke::new(1.0, border_strong),
        fg_stroke: Stroke::new(1.0, text_primary),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };

    // ── Widgets: hovered ─────────────────────────────────────────────────────
    v.widgets.hovered = WidgetVisuals {
        weak_bg_fill: Color32::from_rgb(238, 242, 255), // light blue tint
        bg_fill: Color32::from_rgb(226, 232, 240),
        bg_stroke: Stroke::new(1.0, primary_hover),
        fg_stroke: Stroke::new(1.5, text_primary),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };

    // ── Widgets: active ──────────────────────────────────────────────────────
    v.widgets.active = WidgetVisuals {
        weak_bg_fill: Color32::from_rgb(224, 231, 255),
        bg_fill: Color32::from_rgb(196, 210, 245),
        bg_stroke: Stroke::new(1.0, primary_active),
        fg_stroke: Stroke::new(2.0, text_primary),
        rounding: Rounding::same(4.0),
        expansion: 0.0,
    };

    // ── Widgets: open menu ───────────────────────────────────────────────────
    v.widgets.open = WidgetVisuals {
        weak_bg_fill: Color32::from_rgb(238, 242, 255),
        bg_fill: bg_panel,
        bg_stroke: Stroke::new(1.0, primary_hover),
        fg_stroke: Stroke::new(1.0, text_primary),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };

    // ── Misc ─────────────────────────────────────────────────────────────────
    v.button_frame = true;
    v.collapsing_header_frame = false;
    v.indent_has_left_vline = true;
    v.striped = false;
    v.slider_trailing_fill = true;
    v.interact_cursor = Some(CursorIcon::PointingHand);
    v.text_cursor = TextCursorStyle {
        stroke: Stroke::new(2.0, primary),
        ..TextCursorStyle::default()
    };
}

// ── Dark theme ───────────────────────────────────────────────────────────────

fn apply_dark_visuals(style: &mut Style) {
    let v = &mut style.visuals;
    *v = Visuals::dark();

    let primary = Color32::from_rgb(96, 165, 250); // #60A5FA
    let primary_hover = Color32::from_rgb(85, 141, 224); // #558DE0
    let primary_active = Color32::from_rgb(74, 120, 200); // #4A78C8
    let text_primary = Color32::from_rgb(229, 231, 235); // #E5E7EB
    let text_secondary = Color32::from_rgb(156, 163, 175); // #9CA3AF
    let bg_window = Color32::from_rgb(30, 31, 51); // #1E1F33
    let bg_panel = Color32::from_rgb(26, 27, 47); // #1A1B2F
    let bg_faint = Color32::from_rgb(34, 35, 58); // #22233A
    let bg_extreme = Color32::from_rgb(19, 20, 34); // #131422
    let border = Color32::from_rgb(49, 50, 74); // #31324A
    let border_strong = Color32::from_rgb(66, 68, 94); // #42445E

    v.dark_mode = true;

    // ── Text & hyperlinks ───────────────────────────────────────────────────
    v.override_text_color = Some(text_primary);
    v.hyperlink_color = primary;
    v.warn_fg_color = Color32::from_rgb(251, 191, 36); // #FBBF24  amber
    v.error_fg_color = Color32::from_rgb(248, 113, 113); // #F87171  red
    v.code_bg_color = Color32::from_rgb(45, 46, 68);
    v.faint_bg_color = bg_faint;
    v.extreme_bg_color = bg_extreme;

    // ── Selection ────────────────────────────────────────────────────────────
    v.selection.bg_fill = primary;
    v.selection.stroke = Stroke::new(1.0, primary);

    // ── Window ───────────────────────────────────────────────────────────────
    v.window_rounding = Rounding::same(12.0);
    v.window_shadow = Shadow {
        offset: Vec2::new(0.0, 4.0),
        blur: 24.0,
        spread: 0.0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 60),
    };
    v.window_fill = bg_window;
    v.window_stroke = Stroke::new(1.0, border);
    v.window_highlight_topmost = true;

    // ── Menu ─────────────────────────────────────────────────────────────────
    v.menu_rounding = Rounding::same(8.0);
    v.popup_shadow = Shadow {
        offset: Vec2::new(0.0, 10.0),
        blur: 25.0,
        spread: 0.0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 60),
    };

    // ── Panel fill ───────────────────────────────────────────────────────────
    v.panel_fill = bg_panel;

    // ── Widgets: non-interactive ─────────────────────────────────────────────
    v.widgets.noninteractive = WidgetVisuals {
        weak_bg_fill: bg_panel,
        bg_fill: bg_panel,
        bg_stroke: Stroke::new(1.0, border),
        fg_stroke: Stroke::new(1.0, text_secondary),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };

    // ── Widgets: inactive ────────────────────────────────────────────────────
    v.widgets.inactive = WidgetVisuals {
        weak_bg_fill: Color32::from_rgb(42, 43, 65), // #2A2B41
        bg_fill: Color32::from_rgb(49, 50, 74),
        bg_stroke: Stroke::new(1.0, border_strong),
        fg_stroke: Stroke::new(1.0, text_primary),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };

    // ── Widgets: hovered ─────────────────────────────────────────────────────
    v.widgets.hovered = WidgetVisuals {
        weak_bg_fill: Color32::from_rgb(34, 40, 65), // subtle blue tint
        bg_fill: Color32::from_rgb(49, 50, 74),
        bg_stroke: Stroke::new(1.0, primary_hover),
        fg_stroke: Stroke::new(1.5, text_primary),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };

    // ── Widgets: active ──────────────────────────────────────────────────────
    v.widgets.active = WidgetVisuals {
        weak_bg_fill: Color32::from_rgb(30, 36, 60),
        bg_fill: Color32::from_rgb(55, 56, 84),
        bg_stroke: Stroke::new(1.0, primary_active),
        fg_stroke: Stroke::new(2.0, text_primary),
        rounding: Rounding::same(4.0),
        expansion: 0.0,
    };

    // ── Widgets: open menu ───────────────────────────────────────────────────
    v.widgets.open = WidgetVisuals {
        weak_bg_fill: Color32::from_rgb(34, 40, 65),
        bg_fill: bg_panel,
        bg_stroke: Stroke::new(1.0, primary_hover),
        fg_stroke: Stroke::new(1.0, text_primary),
        rounding: Rounding::same(6.0),
        expansion: 0.0,
    };

    // ── Misc ─────────────────────────────────────────────────────────────────
    v.button_frame = true;
    v.collapsing_header_frame = false;
    v.indent_has_left_vline = true;
    v.striped = false;
    v.slider_trailing_fill = true;
    v.interact_cursor = Some(CursorIcon::PointingHand);
    v.text_cursor = TextCursorStyle {
        stroke: Stroke::new(2.0, primary),
        ..TextCursorStyle::default()
    };
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
                primary: Color32::from_rgb(96, 165, 250),
                success: Color32::from_rgb(52, 211, 153),
                warning: Color32::from_rgb(251, 191, 36),
                danger: Color32::from_rgb(248, 113, 113),
                info: Color32::from_rgb(147, 197, 253),
            }
        } else {
            AccentColors {
                primary: Color32::from_rgb(67, 97, 238),
                success: Color32::from_rgb(16, 185, 129),
                warning: Color32::from_rgb(245, 158, 11),
                danger: Color32::from_rgb(239, 68, 68),
                info: Color32::from_rgb(59, 130, 246),
            }
        }
    }
}

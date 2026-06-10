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

//! A keyboard-driven command palette (Ctrl/Cmd + K).
//!
//! Provides instant fuzzy access to navigation and common actions, in the
//! spirit of the command palettes found in modern editors. The palette is a
//! self-contained overlay: it returns a [`PaletteAction`] that the host app
//! applies, keeping coupling minimal.

use std::time::Instant;

use crate::app::Module;
use crate::i18n::Language;
use crate::theme::{self, AccentColors, ThemeMode};

/// An action emitted by the palette for the host application to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteAction {
    Goto(Module),
    NewMatch,
    SetTheme(ThemeMode),
    SetLanguage(Language),
    ShowAbout,
}

/// A single selectable command entry.
struct Command {
    icon: &'static str,
    label: String,
    /// Extra (lowercased) keywords used for matching, e.g. English aliases.
    keywords: &'static str,
    action: PaletteAction,
}

/// Palette state, owned by the host app.
pub struct CommandPalette {
    pub open: bool,
    query: String,
    selected: usize,
    opened_at: Instant,
    focus_pending: bool,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        CommandPalette {
            open: false,
            query: String::new(),
            selected: 0,
            opened_at: Instant::now(),
            focus_pending: false,
        }
    }

    /// Open the palette, resetting its query and selection.
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.opened_at = Instant::now();
        self.focus_pending = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Render the palette overlay. Returns an action when the user picks one.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        lang: Language,
        accent: &AccentColors,
    ) -> Option<PaletteAction> {
        if !self.open {
            return None;
        }

        let commands = build_commands(lang);

        // Filter by query (matches label or keywords, case-insensitive).
        let q = self.query.trim().to_lowercase();
        let filtered: Vec<usize> = commands
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                q.is_empty()
                    || c.label.to_lowercase().contains(q.as_str())
                    || c.keywords.contains(q.as_str())
            })
            .map(|(i, _)| i)
            .collect();

        // Clamp selection to the filtered range.
        if self.selected >= filtered.len() {
            self.selected = filtered.len().saturating_sub(1);
        }

        // ── Keyboard navigation ──
        let mut chosen: Option<PaletteAction> = None;
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.open = false;
            }
            if i.key_pressed(egui::Key::ArrowDown) && !filtered.is_empty() {
                self.selected = (self.selected + 1) % filtered.len();
            }
            if i.key_pressed(egui::Key::ArrowUp) && !filtered.is_empty() {
                self.selected = (self.selected + filtered.len() - 1) % filtered.len();
            }
            if i.key_pressed(egui::Key::Enter) {
                if let Some(&idx) = filtered.get(self.selected) {
                    chosen = Some(commands[idx].action);
                }
            }
        });

        // ── Appearance animation (fade + slide-down) ──
        let raw = (self.opened_at.elapsed().as_secs_f32() / 0.16).clamp(0.0, 1.0);
        let anim = 1.0 - (1.0 - raw).powi(3);
        if anim < 1.0 {
            ctx.request_repaint();
        }

        let visuals = ctx.style().visuals.clone();
        let screen = ctx.screen_rect();

        // ── Dimmed click-catcher backdrop ──
        egui::Area::new("palette_backdrop".into())
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let resp = ui.allocate_rect(screen, egui::Sense::click());
                ui.painter().rect_filled(
                    screen,
                    egui::Rounding::ZERO,
                    egui::Color32::from_black_alpha((90.0 * anim) as u8),
                );
                if resp.clicked() {
                    self.open = false;
                }
            });

        // ── Palette card ──
        let card_w = 540.0_f32.min(screen.width() - 48.0);
        let slide = (1.0 - anim) * 12.0;
        egui::Area::new("palette_card".into())
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 96.0 + slide))
            .show(ctx, |ui| {
                ui.set_opacity(anim);
                egui::Frame {
                    fill: visuals.window_fill,
                    rounding: egui::Rounding::same(12.0),
                    stroke: egui::Stroke::new(1.0, visuals.window_stroke.color),
                    shadow: egui::Shadow {
                        offset: egui::Vec2::new(0.0, 10.0),
                        blur: 32.0,
                        spread: 0.0,
                        color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 55),
                    },
                    inner_margin: egui::Margin::same(10.0),
                    outer_margin: egui::Margin::ZERO,
                }
                .show(ui, |ui| {
                    ui.set_width(card_w);

                    // Search row with a leading glyph.
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("\u{1F50D}")
                                .size(15.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                        ui.add_space(4.0);
                        let edit = egui::TextEdit::singleline(&mut self.query)
                            .hint_text(tr(lang, "搜索命令…", "Search commands…"))
                            .desired_width(f32::INFINITY)
                            .frame(false)
                            .font(egui::FontId::proportional(16.0));
                        let resp = ui.add(edit);
                        if self.focus_pending {
                            resp.request_focus();
                            self.focus_pending = false;
                        }
                        if resp.changed() {
                            self.selected = 0;
                        }
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    if filtered.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new(tr(lang, "没有匹配的命令", "No matching commands"))
                                    .color(ui.visuals().weak_text_color()),
                            );
                            ui.add_space(16.0);
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(360.0)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                for (row, &idx) in filtered.iter().enumerate() {
                                    let cmd = &commands[idx];
                                    let is_sel = row == self.selected;
                                    if self.draw_row(ui, cmd, is_sel, accent) {
                                        chosen = Some(cmd.action);
                                    }
                                }
                            });
                    }

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(tr(
                                lang,
                                "↑↓ 选择   ⏎ 执行   Esc 关闭",
                                "↑↓ navigate   ⏎ run   Esc close",
                            ))
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                        );
                    });
                });
            });

        if chosen.is_some() {
            self.open = false;
        }
        chosen
    }

    /// Draw one command row; returns true if it was clicked.
    fn draw_row(
        &self,
        ui: &mut egui::Ui,
        cmd: &Command,
        is_sel: bool,
        accent: &AccentColors,
    ) -> bool {
        let (rect, resp) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 38.0),
            egui::Sense::click(),
        );
        let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

        let hover_t = ui
            .ctx()
            .animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.10);
        let highlight = if is_sel { 1.0 } else { hover_t * 0.55 };

        let is_dark = ui.visuals().dark_mode;
        let tint = if is_dark { 0.22 } else { 0.13 };
        let painter = ui.painter().clone();
        if highlight > 0.001 {
            painter.rect_filled(
                rect,
                egui::Rounding::same(8.0),
                accent.primary.gamma_multiply(tint * highlight),
            );
        }

        let text_color =
            theme::lerp_color(ui.visuals().text_color(), accent.primary, highlight);
        painter.text(
            egui::pos2(rect.left() + 18.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            cmd.icon,
            egui::FontId::proportional(15.0),
            text_color,
        );
        painter.text(
            egui::pos2(rect.left() + 44.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &cmd.label,
            egui::FontId::proportional(14.5),
            text_color,
        );

        resp.clicked()
    }
}

/// Pick a localized string (Chinese vs. English fallback).
fn tr(lang: Language, zh: &'static str, en: &'static str) -> &'static str {
    if lang == Language::ChineseSimplified {
        zh
    } else {
        en
    }
}

/// Build the full command list for the given language.
fn build_commands(lang: Language) -> Vec<Command> {
    let l = |zh: &'static str, en: &'static str| tr(lang, zh, en).to_string();

    vec![
        // Navigation
        Command {
            icon: "\u{1F4DD}", // 📝
            label: l("前往：匹配管理", "Go to: Matches"),
            keywords: "matches match manager 匹配",
            action: PaletteAction::Goto(Module::MatchManager),
        },
        Command {
            icon: "\u{1F9E9}", // 🧩
            label: l("前往：包管理", "Go to: Packages"),
            keywords: "packages hub 包",
            action: PaletteAction::Goto(Module::PackageManager),
        },
        Command {
            icon: "\u{2699}", // ⚙
            label: l("前往：设置", "Go to: Settings"),
            keywords: "settings preferences 设置",
            action: PaletteAction::Goto(Module::Settings),
        },
        Command {
            icon: "\u{1F4CA}", // 📊
            label: l("前往：统计", "Go to: Stats"),
            keywords: "stats statistics dashboard 统计",
            action: PaletteAction::Goto(Module::StatsDashboard),
        },
        // Actions
        Command {
            icon: "\u{2795}", // ➕
            label: l("新建匹配", "New match"),
            keywords: "new add match create 新建",
            action: PaletteAction::NewMatch,
        },
        // Theme
        Command {
            icon: "\u{1F5A5}", // 🖥
            label: l("主题：跟随系统", "Theme: Follow system"),
            keywords: "theme system 主题",
            action: PaletteAction::SetTheme(ThemeMode::System),
        },
        Command {
            icon: "\u{2600}", // ☀
            label: l("主题：浅色", "Theme: Light"),
            keywords: "theme light 浅色",
            action: PaletteAction::SetTheme(ThemeMode::Light),
        },
        Command {
            icon: "\u{1F319}", // 🌙
            label: l("主题：深色", "Theme: Dark"),
            keywords: "theme dark 深色",
            action: PaletteAction::SetTheme(ThemeMode::Dark),
        },
        // Language
        Command {
            icon: "\u{1F310}", // 🌐
            label: l("语言：简体中文", "Language: 简体中文"),
            keywords: "language chinese zh 语言 中文",
            action: PaletteAction::SetLanguage(Language::ChineseSimplified),
        },
        Command {
            icon: "\u{1F310}",
            label: l("语言：English", "Language: English"),
            keywords: "language english en 语言",
            action: PaletteAction::SetLanguage(Language::English),
        },
        Command {
            icon: "\u{1F310}",
            label: l("语言：日本語", "Language: 日本語"),
            keywords: "language japanese ja 语言",
            action: PaletteAction::SetLanguage(Language::Japanese),
        },
        // About
        Command {
            icon: "\u{2139}", // ℹ
            label: l("关于", "About"),
            keywords: "about info author 关于",
            action: PaletteAction::ShowAbout,
        },
    ]
}

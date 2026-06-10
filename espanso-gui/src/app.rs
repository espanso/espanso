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

use log::info;

use crate::i18n::{self, Language};
use crate::ipc::IpcClient;
use crate::theme::{self, ThemeMode};

/// Which module is currently shown in the main area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    MatchManager,
    PackageManager,
    Settings,
    StatsDashboard,
}

impl Module {
    pub fn from_str(s: &str) -> Self {
        match s {
            "package" | "packages" => Module::PackageManager,
            "settings" | "setting" => Module::Settings,
            "stats" | "statistics" => Module::StatsDashboard,
            _ => Module::MatchManager,
        }
    }

    pub fn all() -> [Module; 4] {
        [
            Module::MatchManager,
            Module::PackageManager,
            Module::Settings,
            Module::StatsDashboard,
        ]
    }
}

/// Global application state.
pub struct EspansoGuiApp {
    // Paths
    config_dir: Option<PathBuf>,
    runtime_dir: Option<PathBuf>,

    // UI state
    current_module: Module,
    theme_mode: ThemeMode,
    language: Language,

    // IPC
    ipc_client: IpcClient,

    // Module-specific state
    pub match_manager: crate::modules::match_manager::MatchManagerState,
    pub package_manager: crate::modules::package_manager::PackageManagerState,
    pub settings: crate::modules::settings::SettingsState,
    pub stats_dashboard: crate::modules::stats_dashboard::StatsDashboardState,

    // Global UI
    show_about: bool,
    show_theme_menu: bool,
    show_lang_menu: bool,

    // Toast notifications
    toast: Option<Toast>,

    // Worker status
    worker_connected: bool,

    // Animation: timestamp of the last module switch (drives the content fade-in).
    module_changed_at: std::time::Instant,
    // Animation: timestamp the About dialog was opened (drives its fade-in).
    about_opened_at: std::time::Instant,

    // Ctrl/Cmd+K command palette.
    command_palette: crate::command_palette::CommandPalette,
}

struct Toast {
    message: String,
    kind: ToastKind,
    created: std::time::Instant,
}

enum ToastKind {
    Success,
    Error,
    Warning,
    Info,
}

impl EspansoGuiApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        config_dir: Option<PathBuf>,
        runtime_dir: Option<PathBuf>,
        initial_module: Module,
    ) -> Self {
        let lang = Language::detect_system();
        info!("Detected system language: {:?}", lang);

        let config_dir2 = config_dir.clone();

        let mut package_mgr = crate::modules::package_manager::PackageManagerState::new();
        package_mgr.set_paths(
            config_dir.clone(),
            None,
            runtime_dir.clone(),
        );

        let mut stats = crate::modules::stats_dashboard::StatsDashboardState::new();
        stats.set_config_dir(config_dir);

        EspansoGuiApp {
            config_dir: config_dir2.clone(),
            runtime_dir: runtime_dir.clone(),
            current_module: initial_module,
            theme_mode: ThemeMode::System,
            language: lang,
            ipc_client: IpcClient::new(runtime_dir),
            match_manager: crate::modules::match_manager::MatchManagerState::new(
                config_dir2.clone(),
            ),
            package_manager: package_mgr,
            settings: crate::modules::settings::SettingsState::new(config_dir2),
            stats_dashboard: stats,
            show_about: false,
            show_theme_menu: false,
            show_lang_menu: false,
            toast: None,
            worker_connected: false,
            module_changed_at: std::time::Instant::now(),
            about_opened_at: std::time::Instant::now(),
            command_palette: crate::command_palette::CommandPalette::new(),
        }
    }

    /// Switch the active module, recording the time so the content can fade in.
    fn switch_module(&mut self, module: Module) {
        if self.current_module != module {
            self.current_module = module;
            self.module_changed_at = std::time::Instant::now();
        }
    }

    fn show_toast(&mut self, message: String, kind: ToastKind) {
        self.toast = Some(Toast {
            message,
            kind,
            created: std::time::Instant::now(),
        });
    }
}

impl eframe::App for EspansoGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        theme::apply_theme(ctx, self.theme_mode);

        // Try connecting to Worker on first frame
        if !self.worker_connected {
            if let Ok(connected) = self.ipc_client.try_connect() {
                self.worker_connected = connected;
            }
        }

        // Toast timeout (4 seconds)
        if let Some(ref toast) = self.toast {
            if toast.created.elapsed() > std::time::Duration::from_secs(4) {
                self.toast = None;
            }
        }

        let t = i18n::get(self.language);

        // Cache visual style for panel frames
        let visuals = ctx.style().visuals.clone();
        let panel_fill = visuals.panel_fill;
        let window_fill = visuals.window_fill;
        let border = visuals.window_stroke;
        let accent = theme::AccentColors::for_theme(visuals.dark_mode);

        // === Top bar ===
        egui::TopBottomPanel::top("top_bar")
            .frame(egui::Frame {
                fill: window_fill,
                inner_margin: egui::Margin::symmetric(14.0, 8.0),
                stroke: egui::Stroke::new(1.0, border.color),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Brand mark — a small flat tile with a "T". One accent
                    // color only; no gradients.
                    let (logo_rect, _) =
                        ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
                    {
                        let p = ui.painter();
                        p.rect_filled(logo_rect, egui::Rounding::same(6.0), accent.primary);
                        p.text(
                            logo_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "T",
                            egui::FontId::proportional(13.0),
                            egui::Color32::WHITE,
                        );
                    }

                    ui.add_space(8.0);

                    // App heading
                    ui.label(
                        egui::RichText::new("快捷文本输入")
                            .size(16.0)
                            .strong(),
                    );

                    ui.add_space(8.0);

                    // Vertical separator line
                    ui.separator();

                    ui.add_space(8.0);

                    // Worker connection indicator — animated pulsing dot.
                    let connected = self.worker_connected;
                    let pulse = if connected {
                        0.5 + 0.5 * (ui.input(|i| i.time) as f32 * 2.2).sin()
                    } else {
                        0.0
                    };
                    let dot_color = if connected {
                        accent.success
                    } else {
                        egui::Color32::from_rgb(156, 163, 175)
                    };
                    let (dot_rect, _) =
                        ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                    let dc = dot_rect.center();
                    if connected {
                        ui.painter()
                            .circle_filled(dc, 3.5 + 1.5 * pulse, dot_color.gamma_multiply(0.12));
                    }
                    ui.painter().circle_filled(dc, 3.5, dot_color);
                    ui.add_space(2.0);
                    if connected {
                        ui.small(
                            t.common
                                .as_ref()
                                .map_or("Connected", |c| c.worker_connected.as_str()),
                        );
                    } else {
                        ui.small(
                            t.common
                                .as_ref()
                                .map_or("Disconnected", |c| c.worker_disconnected.as_str()),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Theme switcher
                        ui.menu_button(self.theme_mode.label(), |ui| {
                            for mode in ThemeMode::all() {
                                if ui.button(mode.label()).clicked() {
                                    self.theme_mode = mode;
                                    ui.close_menu();
                                }
                            }
                        });

                        ui.add_space(4.0);

                        // Language switcher
                        ui.menu_button(self.language.label(), |ui| {
                            for lang in Language::all() {
                                if ui.button(lang.label()).clicked() {
                                    self.language = lang;
                                    ui.close_menu();
                                }
                            }
                        });

                        ui.add_space(4.0);

                        // About button
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("\u{2139}").size(14.0))
                                    .rounding(egui::Rounding::same(6.0))
                                    .min_size(egui::vec2(32.0, 28.0)),
                            )
                            .clicked()
                        {
                            self.show_about = true;
                            self.about_opened_at = std::time::Instant::now();
                        }

                        ui.add_space(4.0);

                        // Command palette launcher (Ctrl/Cmd + K)
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("\u{1F50D}").size(13.0))
                                    .rounding(egui::Rounding::same(6.0))
                                    .min_size(egui::vec2(32.0, 28.0)),
                            )
                            .on_hover_text("命令面板  ·  Ctrl/⌘ K")
                            .clicked()
                        {
                            self.command_palette.open();
                        }
                    });
                });
            });

        // === Navigation sidebar ===
        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .default_width(180.0)
            .frame(egui::Frame {
                fill: panel_fill,
                inner_margin: egui::Margin::symmetric(8.0, 12.0),
                ..Default::default()
            })
            .show(ctx, |ui| {
                let nav = t.nav.as_ref();
                let modules = [
                    (
                        Module::MatchManager,
                        "\u{1F4DD}", // 📝
                        nav.map_or("Matches", |n| n.match_manager.as_str()),
                    ),
                    (
                        Module::PackageManager,
                        "\u{1F9E9}", // 🧩
                        nav.map_or("Packages", |n| n.package_manager.as_str()),
                    ),
                    (
                        Module::Settings,
                        "\u{2699}", // ⚙
                        nav.map_or("Settings", |n| n.settings.as_str()),
                    ),
                    (
                        Module::StatsDashboard,
                        "\u{1F4CA}", // 📊
                        nav.map_or("Stats", |n| n.stats_dashboard.as_str()),
                    ),
                ];

                ui.add_space(6.0);

                for (module, icon, label) in &modules {
                    let selected = self.current_module == *module;

                    let desired = egui::vec2(ui.available_width(), 38.0);
                    let (rect, resp) =
                        ui.allocate_exact_size(desired, egui::Sense::click());
                    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

                    // Animated states: hover fade and selection cross-fade.
                    let hover_t =
                        ui.ctx().animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.12);
                    let sel_t =
                        ui.ctx().animate_bool_with_time(resp.id.with("sel"), selected, 0.18);

                    let is_dark = ui.visuals().dark_mode;
                    let base_text = ui.visuals().text_color();
                    let painter = ui.painter().clone();

                    // Quiet neutral hover (suppressed as selection takes over).
                    let hov_strength = hover_t * (1.0 - sel_t);
                    if hov_strength > 0.001 {
                        painter.rect_filled(
                            rect,
                            egui::Rounding::same(8.0),
                            base_text.gamma_multiply(0.05 * hov_strength),
                        );
                    }

                    // Selected state: a soft accent tint instead of a solid fill.
                    if sel_t > 0.001 {
                        let tint = if is_dark { 0.20 } else { 0.12 };
                        painter.rect_filled(
                            rect,
                            egui::Rounding::same(8.0),
                            accent.primary.gamma_multiply(tint * sel_t),
                        );
                    }

                    // Icon + label, tweening toward the accent color when selected.
                    let text_color = theme::lerp_color(base_text, accent.primary, sel_t);
                    painter.text(
                        egui::pos2(rect.left() + 16.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        *icon,
                        egui::FontId::proportional(14.0),
                        text_color,
                    );
                    painter.text(
                        egui::pos2(rect.left() + 42.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        *label,
                        egui::FontId::proportional(14.0),
                        text_color,
                    );

                    if resp.clicked() {
                        self.switch_module(*module);
                    }

                    ui.add_space(2.0);
                }

                ui.add_space(16.0);

                // Version info at bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        ui.label(
                            egui::RichText::new("v0.1")
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                        ui.label(
                            egui::RichText::new("·  by yihang_01")
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                    ui.add_space(2.0);
                    ui.separator();
                });
            });

        // === Main content area (with a gentle fade + slide-in on switch) ===
        let elapsed = self.module_changed_at.elapsed().as_secs_f32();
        let raw = (elapsed / 0.20).clamp(0.0, 1.0);
        let fade = 1.0 - (1.0 - raw).powi(3); // ease-out cubic
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_opacity(fade);
            ui.add_space((1.0 - fade) * 8.0); // subtle downward slide
            match self.current_module {
                Module::MatchManager => {
                    crate::modules::match_manager::show(ui, &mut self.match_manager, t);
                }
                Module::PackageManager => {
                    crate::modules::package_manager::show(ui, &mut self.package_manager, t);
                }
                Module::Settings => {
                    crate::modules::settings::show(ui, &mut self.settings, t);
                }
                Module::StatsDashboard => {
                    crate::modules::stats_dashboard::show(
                        ui,
                        &mut self.stats_dashboard,
                        t,
                        &self.ipc_client,
                    );
                }
            }
        });
        if fade < 1.0 {
            ctx.request_repaint();
        }

        // === Toast notification overlay ===
        if let Some(ref toast) = self.toast {
            let (icon, color) = match toast.kind {
                ToastKind::Success => (
                    "\u{2713}", // checkmark
                    egui::Color32::from_rgb(16, 185, 129),
                ),
                ToastKind::Error => (
                    "\u{2717}", // cross
                    egui::Color32::from_rgb(239, 68, 68),
                ),
                ToastKind::Warning => (
                    "\u{26A0}", // warning sign
                    egui::Color32::from_rgb(245, 158, 11),
                ),
                ToastKind::Info => (
                    "\u{2139}", // info
                    egui::Color32::from_rgb(59, 130, 246),
                ),
            };

            let toast_visuals = ctx.style().visuals.clone();

            // Slide-up + fade-in on appear, fade-out shortly before the 4s expiry.
            let life = toast.created.elapsed().as_secs_f32();
            let appear = ((life / 0.25).clamp(0.0, 1.0)).powf(0.6);
            let disappear = ((4.0 - life) / 0.35).clamp(0.0, 1.0);
            let vis = appear.min(disappear);
            let slide = (1.0 - appear) * 16.0;

            egui::Area::new("toast".into())
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -24.0 + slide))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    ui.set_opacity(vis);
                    egui::Frame {
                        fill: toast_visuals.window_fill,
                        rounding: egui::Rounding::same(10.0),
                        stroke: egui::Stroke::new(1.0, toast_visuals.window_stroke.color),
                        shadow: egui::Shadow {
                            offset: egui::Vec2::new(0.0, 4.0),
                            blur: 16.0,
                            spread: 0.0,
                            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 35),
                        },
                        inner_margin: egui::Margin::symmetric(16.0, 12.0),
                        outer_margin: egui::Margin::default(),
                    }
                    .show(ui, |ui| {
                        ui.set_min_width(320.0);
                        ui.horizontal(|ui| {
                            // Colored leading accent bar.
                            let (bar, _) = ui.allocate_exact_size(
                                egui::vec2(3.0, 20.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(bar, egui::Rounding::same(1.5), color);
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(icon).color(color).size(16.0));
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new(&toast.message).size(14.0));
                        });
                    });
                });

            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }

        // === About dialog ===
        if self.show_about {
            let about_raw = (self.about_opened_at.elapsed().as_secs_f32() / 0.18).clamp(0.0, 1.0);
            let about_t = 1.0 - (1.0 - about_raw).powi(3);

            egui::Window::new("关于")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .fixed_size(egui::vec2(340.0, 0.0))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .frame(egui::Frame {
                    fill: window_fill,
                    rounding: egui::Rounding::same(12.0),
                    stroke: egui::Stroke::new(1.0, border.color),
                    shadow: egui::Shadow {
                        offset: egui::Vec2::new(0.0, 8.0),
                        blur: 28.0,
                        spread: 0.0,
                        color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 45),
                    },
                    inner_margin: egui::Margin::symmetric(24.0, 0.0),
                    outer_margin: egui::Margin::ZERO,
                })
                .show(ctx, |ui| {
                    ui.set_opacity(about_t);

                    ui.vertical_centered(|ui| {
                        ui.add_space(28.0);

                        // Flat brand tile.
                        let (logo, _) = ui.allocate_exact_size(
                            egui::vec2(44.0, 44.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(
                            logo,
                            egui::Rounding::same(10.0),
                            accent.primary,
                        );
                        ui.painter().text(
                            logo.center(),
                            egui::Align2::CENTER_CENTER,
                            "T",
                            egui::FontId::proportional(24.0),
                            egui::Color32::WHITE,
                        );

                        ui.add_space(14.0);
                        ui.label(egui::RichText::new("快捷文本输入").size(18.0).strong());

                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("帮你更快地输入常用内容")
                                .size(13.0)
                                .color(ui.visuals().weak_text_color()),
                        );

                        ui.add_space(14.0);

                        // Author row
                        ui.horizontal(|ui| {
                            ui.add_space(
                                (ui.available_width() - 150.0).max(0.0) / 2.0,
                            );
                            ui.label(
                                egui::RichText::new("作者")
                                    .size(13.0)
                                    .color(ui.visuals().weak_text_color()),
                            );
                            ui.add_space(8.0);
                            ui.hyperlink_to(
                                egui::RichText::new("yihang_01").size(14.0).strong(),
                                "https://github.com/10yihang",
                            );
                        });

                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(format!("版本 v{}", env!("CARGO_PKG_VERSION")))
                                .size(12.0)
                                .color(ui.visuals().weak_text_color()),
                        );

                        ui.add_space(18.0);

                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("关闭")
                                        .size(14.0)
                                        .color(egui::Color32::WHITE),
                                )
                                .fill(accent.primary)
                                .min_size(egui::vec2(120.0, 34.0))
                                .rounding(egui::Rounding::same(9.0)),
                            )
                            .clicked()
                        {
                            self.show_about = false;
                        }

                        ui.add_space(16.0);
                    });
                });

            if about_t < 1.0 {
                ctx.request_repaint();
            }
        }

        // === Command palette (Ctrl/Cmd + K) ===
        if let Some(action) = self.command_palette.show(ctx, self.language, &accent) {
            use crate::command_palette::PaletteAction;
            match action {
                PaletteAction::Goto(module) => self.switch_module(module),
                PaletteAction::NewMatch => {
                    self.match_manager.show_editor = true;
                    self.match_manager.editing_match_id = None;
                    self.switch_module(Module::MatchManager);
                }
                PaletteAction::SetTheme(mode) => self.theme_mode = mode,
                PaletteAction::SetLanguage(lang) => self.language = lang,
                PaletteAction::ShowAbout => {
                    self.show_about = true;
                    self.about_opened_at = std::time::Instant::now();
                }
            }
        }

        // Keep animating while the worker is connected (for the pulsing dot).
        if self.worker_connected {
            ctx.request_repaint_after(std::time::Duration::from_millis(40));
        }

        // === Keybindings ===
        ctx.input(|i| {
            // Ctrl/Cmd+K: toggle the command palette.
            if i.modifiers.command && i.key_pressed(egui::Key::K) {
                self.command_palette.toggle();
            }

            // Ctrl+1..5: switch modules
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::Num1) {
                    self.switch_module(Module::MatchManager);
                }
                if i.key_pressed(egui::Key::Num2) {
                    self.switch_module(Module::PackageManager);
                }
                if i.key_pressed(egui::Key::Num3) {
                    self.switch_module(Module::Settings);
                }
                if i.key_pressed(egui::Key::Num5) {
                    self.switch_module(Module::StatsDashboard);
                }
                // Ctrl+N: new match
                if i.key_pressed(egui::Key::N) {
                    self.match_manager.show_editor = true;
                    self.match_manager.editing_match_id = None;
                    self.switch_module(Module::MatchManager);
                }
            }
        });
    }
}

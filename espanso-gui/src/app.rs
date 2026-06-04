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

                    // Worker connection indicator
                    if self.worker_connected {
                        ui.label(
                            egui::RichText::new("\u{25CF}")
                                .color(egui::Color32::from_rgb(16, 185, 129))
                                .size(12.0),
                        );
                        ui.small(
                            t.common
                                .as_ref()
                                .map_or("Connected", |c| c.worker_connected.as_str()),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("\u{25CF}")
                                .color(egui::Color32::from_rgb(156, 163, 175))
                                .size(12.0),
                        );
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
                        nav.map_or("Matches", |n| n.match_manager.as_str()),
                    ),
                    (
                        Module::PackageManager,
                        nav.map_or("Packages", |n| n.package_manager.as_str()),
                    ),
                    (
                        Module::Settings,
                        nav.map_or("Settings", |n| n.settings.as_str()),
                    ),
                    (
                        Module::StatsDashboard,
                        nav.map_or("Stats", |n| n.stats_dashboard.as_str()),
                    ),
                ];

                ui.add_space(4.0);

                for (module, label) in &modules {
                    let selected = self.current_module == *module;

                    let text_color = if selected {
                        egui::Color32::WHITE
                    } else {
                        ui.visuals().text_color()
                    };

                    let mut button =
                        egui::Button::new(egui::RichText::new(*label).size(14.0).color(text_color))
                            .min_size(egui::vec2(ui.available_width(), 36.0))
                            .rounding(egui::Rounding::same(8.0));

                    if selected {
                        button = button
                            .fill(ui.visuals().selection.bg_fill)
                            .stroke(egui::Stroke::NONE);
                    }

                    if ui.add(button).clicked() {
                        self.current_module = *module;
                    }

                    ui.add_space(2.0);
                }

                ui.add_space(16.0);

                // Version info at bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.separator();
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        ui.label(
                            egui::RichText::new("v0.1")
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                        );
                    });
                });
            });

        // === Main content area ===
        egui::CentralPanel::default().show(ctx, |ui| match self.current_module {
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
        });

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

            egui::Area::new("toast".into())
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -24.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame {
                        fill: toast_visuals.window_fill,
                        rounding: egui::Rounding::same(10.0),
                        stroke: egui::Stroke::new(1.0, toast_visuals.window_stroke.color),
                        shadow: egui::Shadow {
                            offset: egui::Vec2::new(0.0, 4.0),
                            blur: 16.0,
                            spread: 0.0,
                            color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 40),
                        },
                        inner_margin: egui::Margin::symmetric(18.0, 12.0),
                        outer_margin: egui::Margin::default(),
                    }
                    .show(ui, |ui| {
                        ui.set_min_width(320.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(icon).color(color).size(16.0));
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new(&toast.message).size(14.0));
                        });
                    });
                });

            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // === About dialog ===
        if self.show_about {
            egui::Window::new("关于")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);

                        ui.label(egui::RichText::new("快捷文本输入").size(22.0).strong());

                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new("帮你更快地输入常用内容")
                                .size(13.0)
                                .color(ui.visuals().weak_text_color()),
                        );

                        ui.add_space(16.0);

                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("关闭").size(14.0))
                                    .min_size(egui::vec2(100.0, 32.0))
                                    .rounding(egui::Rounding::same(8.0)),
                            )
                            .clicked()
                        {
                            self.show_about = false;
                        }

                        ui.add_space(8.0);
                    });
                });
        }

        // === Keybindings ===
        ctx.input(|i| {
            // Ctrl+1..5: switch modules
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::Num1) {
                    self.current_module = Module::MatchManager;
                }
                if i.key_pressed(egui::Key::Num2) {
                    self.current_module = Module::PackageManager;
                }
                if i.key_pressed(egui::Key::Num3) {
                    self.current_module = Module::Settings;
                }
                if i.key_pressed(egui::Key::Num5) {
                    self.current_module = Module::StatsDashboard;
                }
                // Ctrl+N: new match
                if i.key_pressed(egui::Key::N) {
                    self.match_manager.show_editor = true;
                    self.match_manager.editing_match_id = None;
                    self.current_module = Module::MatchManager;
                }
            }
        });
    }
}

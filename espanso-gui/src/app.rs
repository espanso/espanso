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

use crate::backend::config_io;
use crate::i18n::{self, Language};
use crate::ipc::IpcClient;
use crate::theme::{self, ThemeMode};

/// Which module is currently shown in the main area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    MatchManager,
    PackageManager,
    Settings,
    TriggerTester,
    StatsDashboard,
}

impl Module {
    pub fn from_str(s: &str) -> Self {
        match s {
            "package" | "packages" => Module::PackageManager,
            "settings" | "setting" => Module::Settings,
            "test" | "tester" | "trigger" => Module::TriggerTester,
            "stats" | "statistics" => Module::StatsDashboard,
            _ => Module::MatchManager,
        }
    }

    pub fn all() -> [Module; 5] {
        [
            Module::MatchManager,
            Module::PackageManager,
            Module::Settings,
            Module::TriggerTester,
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
    pub trigger_tester: crate::modules::trigger_tester::TriggerTesterState,
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

        EspansoGuiApp {
            config_dir: config_dir.clone(),
            runtime_dir: runtime_dir.clone(),
            current_module: initial_module,
            theme_mode: ThemeMode::System,
            language: lang,
            ipc_client: IpcClient::new(runtime_dir),
            match_manager: crate::modules::match_manager::MatchManagerState::new(config_dir.clone()),
            package_manager: crate::modules::package_manager::PackageManagerState::new(),
            settings: crate::modules::settings::SettingsState::new(config_dir),
            trigger_tester: crate::modules::trigger_tester::TriggerTesterState::new(),
            stats_dashboard: crate::modules::stats_dashboard::StatsDashboardState::new(),
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

        // === Top bar ===
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("espanso");

                ui.separator();

                // Worker connection indicator
                if self.worker_connected {
                    ui.label(
                        egui::RichText::new("●")
                            .color(egui::Color32::GREEN)
                            .size(12.0),
                    );
                    ui.small(t.common.as_ref().map_or("Connected", |c| c.worker_connected.as_str()));
                } else {
                    ui.label(
                        egui::RichText::new("●")
                            .color(egui::Color32::GRAY)
                            .size(12.0),
                    );
                    ui.small(t.common.as_ref().map_or("Disconnected", |c| c.worker_disconnected.as_str()));
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

                    // Language switcher
                    ui.menu_button(self.language.label(), |ui| {
                        for lang in Language::all() {
                            if ui.button(lang.label()).clicked() {
                                self.language = lang;
                                ui.close_menu();
                            }
                        }
                    });

                    // About button
                    if ui.button("ℹ").clicked() {
                        self.show_about = true;
                    }
                });
            });
        });

        // === Navigation sidebar ===
        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                let nav = t.nav.as_ref();
                let modules = [
                    (Module::MatchManager, nav.map_or("Matches", |n| n.match_manager.as_str())),
                    (
                        Module::PackageManager,
                        nav.map_or("Packages", |n| n.package_manager.as_str()),
                    ),
                    (Module::Settings, nav.map_or("Settings", |n| n.settings.as_str())),
                    (
                        Module::TriggerTester,
                        nav.map_or("Tester", |n| n.trigger_tester.as_str()),
                    ),
                    (
                        Module::StatsDashboard,
                        nav.map_or("Stats", |n| n.stats_dashboard.as_str()),
                    ),
                ];

                for (module, label) in &modules {
                    let selected = self.current_module == *module;
                    let clicked = ui.selectable_label(selected, *label).clicked();
                    if clicked {
                        self.current_module = *module;
                    }
                }

                ui.add_space(16.0);

                // Version info at bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        ui.small(
                            t.common
                                .as_ref()
                                .map_or("GUI v0.1", |c| c.version.as_str())
                                .replace("{}", "0.1"),
                        );
                    });
                });
            });

        // === Main content area ===
        egui::CentralPanel::default().show(ctx, |ui| {
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
                Module::TriggerTester => {
                    crate::modules::trigger_tester::show(
                        ui,
                        &mut self.trigger_tester,
                        t,
                        &self.ipc_client,
                    );
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

        // === Toast notification overlay ===
        if let Some(ref toast) = self.toast {
            let color = match toast.kind {
                ToastKind::Success => egui::Color32::from_rgb(72, 199, 142),
                ToastKind::Error => egui::Color32::from_rgb(255, 107, 107),
                ToastKind::Warning => egui::Color32::from_rgb(255, 183, 77),
                ToastKind::Info => egui::Color32::from_rgb(129, 178, 255),
            };

            egui::Area::new("toast".into())
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -20.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(300.0);
                        ui.colored_label(color, &toast.message);
                    });
                });

            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // === About dialog ===
        if self.show_about {
            egui::Window::new("About espanso")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("espanso");
                        ui.label("A Privacy-first, Cross-platform Text Expander");
                        ui.add_space(8.0);
                        ui.label(format!(
                            "espanso GUI v{}\nBuilt with egui",
                            env!("CARGO_PKG_VERSION")
                        ));
                        ui.add_space(8.0);
                        ui.label("© 2019-2021 Federico Terzi and contributors");
                        ui.hyperlink("https://espanso.org");
                        ui.add_space(8.0);
                        if ui.button("Close").clicked() {
                            self.show_about = false;
                        }
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
                if i.key_pressed(egui::Key::Num4) {
                    self.current_module = Module::TriggerTester;
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

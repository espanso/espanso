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
use std::sync::mpsc;

use crate::backend::package_io::{
    poll_bg_result, start_check_updates, start_install, start_list_installed,
    start_load_hub_index, start_uninstall, BgOpResult, HubPackageInfo, InstalledPackageInfo,
};
use crate::i18n::Translations;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageTab {
    Installed,
    Hub,
    Updates,
}

enum LoadState {
    Idle,
    Loading,
    Loaded,
    Error(String),
}

pub struct PackageManagerState {
    active_tab: PackageTab,
    search_query: String,
    packages_dir: Option<PathBuf>,
    runtime_dir: Option<PathBuf>,
    // Cached data
    installed: Vec<InstalledPackageInfo>,
    hub_packages: Vec<HubPackageInfo>,
    updates: Vec<(InstalledPackageInfo, String)>,
    // Async loading
    hub_load_state: LoadState,
    installed_load_state: LoadState,
    updates_load_state: LoadState,
    op_state: LoadState, // for install/uninstall
    // Background channels
    hub_rx: Option<mpsc::Receiver<BgOpResult>>,
    installed_rx: Option<mpsc::Receiver<BgOpResult>>,
    update_rx: Option<mpsc::Receiver<BgOpResult>>,
    op_rx: Option<mpsc::Receiver<BgOpResult>>,
}

impl PackageManagerState {
    pub fn new() -> Self {
        PackageManagerState {
            active_tab: PackageTab::Installed,
            search_query: String::new(),
            packages_dir: None,
            runtime_dir: None,
            installed: Vec::new(),
            hub_packages: Vec::new(),
            updates: Vec::new(),
            hub_load_state: LoadState::Idle,
            installed_load_state: LoadState::Idle,
            updates_load_state: LoadState::Idle,
            op_state: LoadState::Idle,
            hub_rx: None,
            installed_rx: None,
            update_rx: None,
            op_rx: None,
        }
    }

    pub fn set_paths(&mut self, _config_dir: Option<PathBuf>, packages_dir: Option<PathBuf>, runtime_dir: Option<PathBuf>) {
        self.packages_dir = packages_dir.or_else(|| {
            _config_dir.as_ref().map(|d| d.join("match").join("packages"))
        });
        self.runtime_dir = runtime_dir;
    }

    fn poll_tasks(&mut self) {
        // Poll Hub
        if let Some(ref rx) = self.hub_rx {
            match poll_bg_result(rx) {
                Some(BgOpResult::HubIndexLoaded(pkgs)) => {
                    self.hub_packages = pkgs;
                    self.hub_load_state = LoadState::Loaded;
                    self.hub_rx = None;
                }
                Some(BgOpResult::Error(e)) => {
                    self.hub_load_state = LoadState::Error(e);
                    self.hub_rx = None;
                }
                _ => {}
            }
        }
        // Poll installed
        if let Some(ref rx) = self.installed_rx {
            match poll_bg_result(rx) {
                Some(BgOpResult::InstalledList(list)) => {
                    self.installed = list;
                    self.installed_load_state = LoadState::Loaded;
                    self.installed_rx = None;
                }
                Some(BgOpResult::Error(e)) => {
                    self.installed_load_state = LoadState::Error(e);
                    self.installed_rx = None;
                }
                _ => {}
            }
        }
        // Poll updates
        if let Some(ref rx) = self.update_rx {
            match poll_bg_result(rx) {
                Some(BgOpResult::UpdatesCheck(list)) => {
                    self.updates = list;
                    self.updates_load_state = LoadState::Loaded;
                    self.update_rx = None;
                }
                Some(BgOpResult::Error(e)) => {
                    self.updates_load_state = LoadState::Error(e);
                    self.update_rx = None;
                }
                _ => {}
            }
        }
        // Poll op
        if let Some(ref rx) = self.op_rx {
            match poll_bg_result(rx) {
                Some(BgOpResult::InstallDone(_) | BgOpResult::UninstallDone(_)) => {
                    self.op_state = LoadState::Loaded;
                    self.op_rx = None;
                    // Refresh installed list
                    self.reload_installed();
                    // Refresh updates
                    self.reload_updates();
                }
                Some(BgOpResult::Error(e)) => {
                    self.op_state = LoadState::Error(e);
                    self.op_rx = None;
                }
                _ => {}
            }
        }
    }

    fn reload_installed(&mut self) {
        if self.packages_dir.is_some() && self.installed_rx.is_none() {
            self.installed_load_state = LoadState::Loading;
            self.installed_rx = Some(start_list_installed(self.packages_dir.clone().unwrap()));
        }
    }

    fn reload_hub(&mut self) {
        if self.runtime_dir.is_some() && self.hub_rx.is_none() {
            self.hub_load_state = LoadState::Loading;
            self.hub_rx = Some(start_load_hub_index(self.runtime_dir.clone().unwrap()));
        }
    }

    fn reload_updates(&mut self) {
        if self.packages_dir.is_some() && self.runtime_dir.is_some() && self.update_rx.is_none() {
            self.updates_load_state = LoadState::Loading;
            self.update_rx = Some(start_check_updates(
                self.packages_dir.clone().unwrap(),
                self.runtime_dir.clone().unwrap(),
            ));
        }
    }

    fn do_install(&mut self, name: &str) {
        if self.packages_dir.is_some() && self.runtime_dir.is_some() && self.op_rx.is_none() {
            self.op_state = LoadState::Loading;
            self.op_rx = Some(start_install(
                self.packages_dir.clone().unwrap(),
                self.runtime_dir.clone().unwrap(),
                name.to_string(),
            ));
        }
    }

    fn do_uninstall(&mut self, name: &str) {
        if self.packages_dir.is_some() && self.op_rx.is_none() {
            self.op_state = LoadState::Loading;
            self.op_rx = Some(start_uninstall(
                self.packages_dir.clone().unwrap(),
                name.to_string(),
            ));
        }
    }
}

pub fn show(ui: &mut egui::Ui, state: &mut PackageManagerState, t: &Translations) {
    let pm_wrapper = t.package_manager.clone();
    let pm = pm_wrapper.as_ref();

    // Poll background tasks every frame
    state.poll_tasks();

    ui.vertical(|ui| {
        ui.heading(pm.map_or("Package Manager", |p| p.title.as_str()));
        ui.add_space(8.0);

        // Tabs
        ui.horizontal(|ui| {
            let installed_label = if state.installed.is_empty() {
                pm.map_or("Installed", |p| p.tab_installed.as_str()).replace("{}", "0")
            } else {
                let label = pm.map_or("Installed", |p| p.tab_installed.as_str());
                label.replace("{}", &state.installed.len().to_string())
            };
            let updates_label = if state.updates.is_empty() {
                pm.map_or("Updates", |p| p.tab_updates.as_str()).replace("{}", "0")
            } else {
                let label = pm.map_or("Updates", |p| p.tab_updates.as_str());
                label.replace("{}", &state.updates.len().to_string())
            };

            for (tab, label) in [
                (PackageTab::Installed, installed_label.as_str()),
                (PackageTab::Hub, pm.map_or("Hub Market", |p| p.tab_hub.as_str())),
                (PackageTab::Updates, updates_label.as_str()),
            ] {
                if ui.selectable_label(state.active_tab == tab, label).clicked() {
                    state.active_tab = tab;
                    match tab {
                        PackageTab::Hub => {
                            if state.hub_packages.is_empty() && matches!(state.hub_load_state, LoadState::Idle) {
                                state.reload_hub();
                            }
                        }
                        PackageTab::Installed => {
                            if state.installed.is_empty() && matches!(state.installed_load_state, LoadState::Idle) {
                                state.reload_installed();
                            }
                        }
                        PackageTab::Updates => {
                            if state.updates.is_empty() && matches!(state.updates_load_state, LoadState::Idle) {
                                state.reload_updates();
                            }
                        }
                    }
                }
            }
        });

        ui.add_space(8.0);

        match state.active_tab {
            PackageTab::Installed => show_installed(ui, state, pm),
            PackageTab::Hub => show_hub(ui, state, pm),
            PackageTab::Updates => show_updates(ui, state, pm),
        }

        // Operation status
        match &state.op_state {
            LoadState::Loading => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Working...");
                });
            }
            LoadState::Error(e) => {
                ui.colored_label(egui::Color32::from_rgb(255, 107, 107), e);
            }
            LoadState::Loaded => {
                ui.colored_label(egui::Color32::from_rgb(72, 199, 142), "Done");
            }
            _ => {}
        }
    });
}

fn show_installed(ui: &mut egui::Ui, state: &mut PackageManagerState, pm: Option<&crate::i18n::PackageManagerTranslations>) {
    match &state.installed_load_state {
        LoadState::Loading => {
            ui.horizontal(|ui| { ui.spinner(); ui.label("Loading..."); });
            return;
        }
        LoadState::Error(e) => {
            ui.colored_label(egui::Color32::from_rgb(255, 107, 107), e);
            if ui.button("Retry").clicked() { state.reload_installed(); }
            return;
        }
        LoadState::Idle => {
            state.reload_installed();
            ui.spinner();
            return;
        }
        _ => {}
    }

    if state.installed.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("No packages installed").size(16.0));
            ui.small("Browse the Hub tab to find packages");
            ui.add_space(8.0);
            if ui.button("Browse Hub").clicked() { state.active_tab = PackageTab::Hub; }
        });
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for pkg in &state.installed.clone() {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.strong(&pkg.title);
                    ui.small(format!("v{} — {}", pkg.version, pkg.description));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(pm.map_or("Uninstall", |p| p.uninstall_button.as_str())).clicked() {
                        state.do_uninstall(&pkg.name);
                    }
                });
            });
            ui.separator();
        }
    });
}

fn show_hub(ui: &mut egui::Ui, state: &mut PackageManagerState, pm: Option<&crate::i18n::PackageManagerTranslations>) {
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.search_query)
                .hint_text(pm.map_or("Search Hub packages...", |p| p.search_placeholder.as_str()))
                .desired_width(300.0),
        );
        if ui.button("Refresh").clicked() { state.reload_hub(); }
    });

    ui.add_space(8.0);

    match &state.hub_load_state {
        LoadState::Loading => {
            ui.horizontal(|ui| { ui.spinner(); ui.label("Fetching Hub index..."); });
            return;
        }
        LoadState::Error(e) => {
            ui.colored_label(egui::Color32::from_rgb(255, 107, 107), e);
            if ui.button("Retry").clicked() { state.reload_hub(); }
            return;
        }
        LoadState::Idle => { return; }
        _ => {}
    }

    if state.hub_packages.is_empty() {
        ui.vertical_centered(|ui| {
            ui.label("No packages on Hub");
            if ui.button("Retry").clicked() { state.reload_hub(); }
        });
        return;
    }

    let query = state.search_query.to_lowercase();
    let filtered: Vec<HubPackageInfo> = state.hub_packages.iter()
        .filter(|p| {
            if query.is_empty() { return true; }
            p.name.to_lowercase().contains(&query)
                || p.title.to_lowercase().contains(&query)
                || p.description.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for pkg in &filtered {
            let installed = state.installed.iter().any(|i| i.name == pkg.name);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.strong(&pkg.title);
                    ui.small(format!("v{} — {} — by {}", pkg.version, pkg.description, pkg.author));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if installed {
                        ui.label("Installed");
                    } else if ui.small_button(pm.map_or("Install", |p| p.install_button.as_str())).clicked() {
                        state.do_install(&pkg.name);
                    }
                });
            });
            ui.separator();
        }
    });
}

fn show_updates(ui: &mut egui::Ui, state: &mut PackageManagerState, pm: Option<&crate::i18n::PackageManagerTranslations>) {
    match &state.updates_load_state {
        LoadState::Loading => { ui.spinner(); return; }
        LoadState::Error(e) => {
            ui.colored_label(egui::Color32::from_rgb(255, 107, 107), e);
            return;
        }
        LoadState::Idle => { state.reload_updates(); ui.spinner(); return; }
        _ => {}
    }

    if state.updates.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.strong("All packages up to date");
        });
        return;
    }

    ui.horizontal(|ui| {
        if ui.button(pm.map_or("Update All", |p| p.update_all_button.as_str())).clicked() {
            for (pkg, _) in state.updates.clone() { state.do_install(&pkg.name); }
        }
    });
    ui.add_space(4.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (pkg, latest) in &state.updates.clone() {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.strong(&pkg.title);
                    ui.small(format!("v{} → v{}", pkg.version, latest));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(pm.map_or("Update", |p| p.update_button.as_str())).clicked() {
                        state.do_install(&pkg.name);
                    }
                });
            });
            ui.separator();
        }
    });
}

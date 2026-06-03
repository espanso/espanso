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

//! Module 2: Package Manager — install, update, and remove packages.

use std::path::PathBuf;

use crate::backend::package_io::{
    check_updates, install_from_hub, list_installed, load_hub_index, uninstall_package,
    HubPackageInfo, InstalledPackageInfo,
};
use crate::i18n::{PackageManagerTranslations, Translations};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageTab {
    Installed,
    Hub,
    Updates,
}

pub struct PackageManagerState {
    active_tab: PackageTab,
    search_query: String,
    config_dir: Option<PathBuf>,
    packages_dir: Option<PathBuf>,
    runtime_dir: Option<PathBuf>,
    // Cached data
    installed: Vec<InstalledPackageInfo>,
    hub_packages: Vec<HubPackageInfo>,
    updates: Vec<(InstalledPackageInfo, String)>,
    needs_reload: bool,
    loading: bool,
    status_message: Option<String>,
}

impl PackageManagerState {
    pub fn new() -> Self {
        PackageManagerState {
            active_tab: PackageTab::Installed,
            search_query: String::new(),
            config_dir: None,
            packages_dir: None,
            runtime_dir: None,
            installed: Vec::new(),
            hub_packages: Vec::new(),
            updates: Vec::new(),
            needs_reload: true,
            loading: false,
            status_message: None,
        }
    }

    pub fn set_paths(
        &mut self,
        config_dir: Option<PathBuf>,
        packages_dir: Option<PathBuf>,
        runtime_dir: Option<PathBuf>,
    ) {
        self.config_dir = config_dir;
        self.packages_dir = packages_dir.or_else(|| {
            self.config_dir
                .as_ref()
                .map(|d| d.join("match").join("packages"))
        });
        self.runtime_dir = runtime_dir;
        self.needs_reload = true;
    }

    fn reload(&mut self) {
        self.loading = true;
        self.installed.clear();
        self.updates.clear();

        if let Some(ref pkgs_dir) = self.packages_dir {
            match list_installed(pkgs_dir) {
                Ok(list) => self.installed = list,
                Err(e) => {
                    self.status_message = Some(format!("Failed to list packages: {}", e));
                }
            }

            if let Some(ref rt_dir) = self.runtime_dir {
                match check_updates(pkgs_dir, rt_dir) {
                    Ok(list) => self.updates = list,
                    Err(_) => {} // Non-critical; updates check can fail silently
                }
            }
        }

        self.loading = false;
        self.needs_reload = false;
    }

    fn reload_hub(&mut self) {
        if self.runtime_dir.is_none() {
            return;
        }
        self.loading = true;
        match load_hub_index(self.runtime_dir.as_ref().unwrap()) {
            Ok(pkgs) => {
                self.hub_packages = pkgs;
                self.hub_packages.sort_by(|a, b| a.name.cmp(&b.name));
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to load Hub: {}", e));
            }
        }
        self.loading = false;
    }

    fn do_install(&mut self, name: &str) {
        let pkgs_dir = match self.packages_dir.clone() {
            Some(d) => d,
            None => {
                self.status_message = Some("No packages directory configured".into());
                return;
            }
        };
        let rt_dir = match self.runtime_dir.clone() {
            Some(d) => d,
            None => {
                self.status_message = Some("No runtime directory configured".into());
                return;
            }
        };

        let name = name.to_string();
        self.loading = true;
        match install_from_hub(&pkgs_dir, &rt_dir, &name) {
            Ok(msg) => {
                self.status_message = Some(msg);
                self.needs_reload = true;
            }
            Err(e) => {
                self.status_message = Some(format!("Install failed: {}", e));
            }
        }
        self.loading = false;
    }

    fn do_uninstall(&mut self, name: &str) {
        let pkgs_dir = match self.packages_dir.clone() {
            Some(d) => d,
            None => return,
        };
        match uninstall_package(&pkgs_dir, name) {
            Ok(msg) => {
                self.status_message = Some(msg);
                self.needs_reload = true;
            }
            Err(e) => {
                self.status_message = Some(format!("Uninstall failed: {}", e));
            }
        }
    }
}

pub fn show(ui: &mut egui::Ui, state: &mut PackageManagerState, t: &Translations) {
    let pm = t.package_manager.as_ref();

    // Ensure data is loaded
    if state.needs_reload && !state.loading {
        state.reload();
    }

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
                    if tab == PackageTab::Hub && state.hub_packages.is_empty() {
                        state.reload_hub();
                    }
                }
            }
        });

        ui.add_space(8.0);

        // Loading spinner
        if state.loading {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading...");
            });
        }

        match state.active_tab {
            PackageTab::Installed => show_installed(ui, state, pm),
            PackageTab::Hub => show_hub(ui, state, pm),
            PackageTab::Updates => show_updates(ui, state, pm),
        }

        // Status message
        if let Some(ref msg) = state.status_message.clone() {
            if msg.starts_with("Failed") || msg.starts_with("Install failed") || msg.starts_with("Uninstall failed") {
                ui.colored_label(egui::Color32::from_rgb(255, 107, 107), msg);
            } else {
                ui.colored_label(egui::Color32::from_rgb(72, 199, 142), msg);
            }
        }
    });
}

fn show_installed(
    ui: &mut egui::Ui,
    state: &mut PackageManagerState,
    pm: Option<&PackageManagerTranslations>,
) {
    if state.installed.is_empty() && !state.loading {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("📦").size(48.0));
            ui.add_space(8.0);
            ui.strong("No packages installed");
            ui.small("Browse the Hub tab to find useful packages");
            ui.add_space(8.0);
            if ui.button("📦 Browse Hub").clicked() {
                state.active_tab = PackageTab::Hub;
                if state.hub_packages.is_empty() {
                    state.reload_hub();
                }
            }
        });
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for pkg in &state.installed.clone() {
            ui.horizontal(|ui| {
                ui.label("📦");
                ui.vertical(|ui| {
                    ui.strong(&pkg.title);
                    let desc = pm.map_or(
                        format!("v{} — {} — by {}", pkg.version, pkg.description, pkg.author),
                        |p| format!(
                            "{} — {} — {} {}",
                            p.version_label.as_str().replace("{}", &pkg.version),
                            &pkg.description,
                            &p.detail_author.as_str().replace("{}", &pkg.author),
                            &pkg.source
                        ),
                    );
                    ui.small(&desc);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(pm.map_or("Uninstall", |p| p.uninstall_button.as_str()))
                        .clicked()
                    {
                        state.do_uninstall(&pkg.name);
                    }
                });
            });
            ui.separator();
        }
    });
}

fn show_hub(
    ui: &mut egui::Ui,
    state: &mut PackageManagerState,
    pm: Option<&PackageManagerTranslations>,
) {
    // Search bar
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.search_query)
                .hint_text(pm.map_or("Search Hub packages...", |p| p.search_placeholder.as_str()))
                .desired_width(300.0),
        );

        if ui.button("🔄 Refresh").clicked() {
            state.hub_packages.clear();
            state.reload_hub();
        }
    });

    ui.add_space(8.0);

    if state.hub_packages.is_empty() && !state.loading {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("🛒").size(48.0));
            ui.add_space(8.0);
            ui.strong("No packages loaded");
            ui.small("Click 'Refresh' to fetch the Hub index");
        });
        return;
    }

    let query = state.search_query.to_lowercase();
    let filtered: Vec<HubPackageInfo> = state
        .hub_packages
        .iter()
        .filter(|p| {
            if query.is_empty() {
                return true;
            }
            p.name.to_lowercase().contains(&query)
                || p.title.to_lowercase().contains(&query)
                || p.description.to_lowercase().contains(&query)
                || p.author.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for pkg in &filtered {
            let installed = state.installed.iter().any(|i| i.name == pkg.name);
            let has_update = state
                .updates
                .iter()
                .any(|(i, _)| i.name == pkg.name);

            ui.horizontal(|ui| {
                ui.label("📦");
                ui.vertical(|ui| {
                    ui.strong(&pkg.title);
                    ui.small(format!("v{} — {} — by {}", pkg.version, pkg.description, pkg.author));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if has_update {
                        if ui.small_button(pm.map_or("Update", |p| p.update_button.as_str())).clicked() {
                            state.do_install(&pkg.name);
                        }
                    } else if installed {
                        ui.small(pm.map_or("Installed", |p| p.installed_label.as_str()));
                    } else if ui
                        .small_button(pm.map_or("Install", |p| p.install_button.as_str()))
                        .clicked()
                    {
                        state.do_install(&pkg.name);
                    }
                });
            });
            ui.separator();
        }
    });
}

fn show_updates(
    ui: &mut egui::Ui,
    state: &mut PackageManagerState,
    pm: Option<&PackageManagerTranslations>,
) {
    if state.updates.is_empty() {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("✅").size(48.0));
            ui.add_space(8.0);
            ui.strong("All packages are up to date");
        });
        return;
    }

    ui.horizontal(|ui| {
        if ui
            .button(pm.map_or("Update All", |p| p.update_all_button.as_str()))
            .clicked()
        {
            for (pkg, _) in state.updates.clone() {
                state.do_install(&pkg.name);
            }
        }
    });

    ui.add_space(4.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (pkg, latest) in &state.updates.clone() {
            ui.horizontal(|ui| {
                ui.label("📦");
                ui.vertical(|ui| {
                    ui.strong(&pkg.title);
                    let update_text = pm.map_or(
                        format!("v{} → v{} available", pkg.version, latest),
                        |p| {
                            p.update_available
                                .replace("{}", &pkg.version)
                                .replace("{}", latest)
                        },
                    );
                    ui.small(&update_text);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(pm.map_or("Update", |p| p.update_button.as_str()))
                        .clicked()
                    {
                        state.do_install(&pkg.name);
                    }
                });
            });
            ui.separator();
        }
    });
}

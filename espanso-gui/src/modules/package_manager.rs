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

use crate::i18n::Translations;

/// Tabs in the package manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageTab {
    Installed,
    Hub,
    Updates,
}

/// State for the package manager module.
pub struct PackageManagerState {
    active_tab: PackageTab,
    search_query: String,
}

impl PackageManagerState {
    pub fn new() -> Self {
        PackageManagerState {
            active_tab: PackageTab::Installed,
            search_query: String::new(),
        }
    }
}

/// Render the package manager module.
pub fn show(ui: &mut egui::Ui, state: &mut PackageManagerState, t: &Translations) {
    let pm = t.package_manager.as_ref();

    ui.vertical(|ui| {
        ui.heading(pm.map_or("Package Manager", |p| p.title.as_str()));

        ui.add_space(8.0);

        // === Tabs ===
        ui.horizontal(|ui| {
            let tabs = [
                (PackageTab::Installed, pm.map_or("Installed", |p| p.tab_installed.as_str())),
                (PackageTab::Hub, pm.map_or("Hub Market", |p| p.tab_hub.as_str())),
                (PackageTab::Updates, pm.map_or("Updates", |p| p.tab_updates.as_str())),
            ];

            for (tab, label) in &tabs {
                let selected = state.active_tab == *tab;
                if ui.selectable_label(selected, *label).clicked() {
                    state.active_tab = *tab;
                }
            }
        });

        ui.add_space(8.0);

        match state.active_tab {
            PackageTab::Installed => show_installed(ui, state, pm),
            PackageTab::Hub => show_hub(ui, state, pm),
            PackageTab::Updates => show_updates(ui, pm),
        }
    });
}

fn show_installed(
    ui: &mut egui::Ui,
    _state: &mut PackageManagerState,
    pm: Option<&crate::i18n::PackageManagerTranslations>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Stub: Show placeholder installed packages
        let packages: Vec<(&str, &str, &str)> = vec![];
        // In full implementation: query from espanso_package::Archiver::list()

        if packages.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("📦").size(48.0));
                ui.add_space(8.0);
                ui.strong("No packages installed");
                ui.small("Browse the Hub to find useful packages");
            });
        }

        for (name, version, desc) in &packages {
            ui.horizontal(|ui| {
                ui.label("📦");
                ui.vertical(|ui| {
                    ui.strong(*name);
                    ui.small(format!("v{} — {}", version, desc));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(pm.map_or("Uninstall", |p| p.uninstall_button.as_str()))
                        .clicked()
                    {
                        // TODO: Uninstall package
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
    pm: Option<&crate::i18n::PackageManagerTranslations>,
) {
    // Search bar
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.search_query)
                .hint_text(pm.map_or("Search Hub packages...", |p| p.search_placeholder.as_str()))
                .desired_width(300.0),
        );
    });

    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Stub: Show placeholder Hub packages
        let hub_packages: Vec<(&str, &str, u32, bool)> = vec![];
        // In full implementation: query from espanso_package::Provider::search()

        if hub_packages.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("🛒").size(48.0));
                ui.add_space(8.0);
                ui.strong("Hub marketplace coming soon");
                ui.small("Package browsing will be available once the Hub integration is complete");
            });
        }

        for (name, desc, stars, is_installed) in &hub_packages {
            ui.horizontal(|ui| {
                ui.label("📦");
                ui.vertical(|ui| {
                    ui.strong(*name);
                    ui.small(format!("⭐ {} — {}", stars, desc));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if *is_installed {
                        ui.small(pm.map_or("Installed", |p| p.installed_label.as_str()));
                    } else {
                        if ui
                            .small_button(pm.map_or("Install", |p| p.install_button.as_str()))
                            .clicked()
                        {
                            // TODO: Install package from Hub
                        }
                    }
                });
            });
            ui.separator();
        }
    });
}

fn show_updates(
    ui: &mut egui::Ui,
    pm: Option<&crate::i18n::PackageManagerTranslations>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Stub: Show placeholder updates
        let updates: Vec<(&str, &str, &str)> = vec![];
        // In full implementation: compare installed versions vs Hub

        if updates.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("✅").size(48.0));
                ui.add_space(8.0);
                ui.strong("All packages are up to date");
            });
        }

        for (name, current, latest) in &updates {
            ui.horizontal(|ui| {
                ui.label("📦");
                ui.vertical(|ui| {
                    ui.strong(*name);
                    let update_text = pm.map_or(
                        format!("v{} → v{} available", current, latest),
                        |p| p.update_available.replace("{}", current).replace("{}", latest),
                    );
                    ui.small(&update_text);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(pm.map_or("Update", |p| p.update_button.as_str()))
                        .clicked()
                    {
                        // TODO: Update package
                    }
                });
            });
            ui.separator();
        }
    });
}

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

//! Module 5: Stats Dashboard — visualize expansion frequency.

use std::fs;
use std::path::PathBuf;

use crate::backend::stats_io::{StatsPeriod, StatsSummary, TriggerStat};
use crate::i18n::Translations;
use crate::ipc::IpcClient;

pub struct StatsDashboardState {
    period: StatsPeriod,
    summary: Option<StatsSummary>,
    stats_enabled: bool,
    is_loading: bool,
    config_dir: Option<PathBuf>,
}

impl StatsDashboardState {
    pub fn new() -> Self {
        StatsDashboardState {
            period: StatsPeriod::Last7Days,
            summary: None,
            stats_enabled: false,
            is_loading: false,
            config_dir: None,
        }
    }

    pub fn set_config_dir(&mut self, dir: Option<PathBuf>) {
        self.config_dir = dir;
        self.check_enabled();
    }

    fn check_enabled(&mut self) {
        if let Some(ref d) = self.config_dir {
            let config_file = d.join("config").join("default.yml");
            if let Ok(content) = fs::read_to_string(&config_file) {
                self.stats_enabled = content
                    .lines()
                    .any(|l| l.trim() == "enabled: true" && content.contains("stats:"));
            }
        }
    }

    pub fn enable_stats(&mut self) {
        let config_dir = match &self.config_dir {
            Some(d) => d.clone(),
            None => return,
        };
        let config_file = config_dir.join("config").join("default.yml");
        let content = fs::read_to_string(&config_file).unwrap_or_default();

        let updated = if content.contains("stats:") {
            // Replace existing stats section
            let lines: Vec<&str> = content.lines().collect();
            let mut out = Vec::new();
            let mut in_stats = false;
            for line in &lines {
                if line.trim_start().starts_with("stats:") {
                    in_stats = true;
                    out.push("stats:".to_string());
                    out.push("  enabled: true".to_string());
                    continue;
                }
                if in_stats {
                    if line.starts_with(' ') || line.starts_with('\t') {
                        continue; // skip old stats fields
                    }
                    in_stats = false;
                }
                out.push(line.to_string());
            }
            out.join("\n")
        } else {
            // Append stats section
            if content.ends_with('\n') {
                format!("{}stats:\n  enabled: true\n", content)
            } else {
                format!("{}\nstats:\n  enabled: true\n", content)
            }
        };

        let _ = fs::create_dir_all(config_file.parent().unwrap());
        let _ = fs::write(&config_file, updated);
        self.stats_enabled = true;
    }

    fn fetch_stats(&mut self, _ipc_client: &IpcClient) {
        self.is_loading = true;
        self.summary = Some(StatsSummary {
            total_expansions: 0,
            active_matches: 0,
            unused_matches: 0,
            top_triggers: Vec::new(),
        });
        self.is_loading = false;
    }
}

pub fn show(
    ui: &mut egui::Ui,
    state: &mut StatsDashboardState,
    t: &Translations,
    ipc_client: &IpcClient,
) {
    let sd = t.stats_dashboard.as_ref();

    ui.vertical(|ui| {
        ui.heading(sd.map_or("Stats Dashboard", |s| s.title.as_str()));
        ui.add_space(8.0);

        // Period selector
        ui.horizontal(|ui| {
            let periods = [
                (StatsPeriod::Last7Days, "Past 7 Days"),
                (StatsPeriod::Last30Days, "This Month"),
                (StatsPeriod::AllTime, "All Time"),
            ];
            for (period, label) in &periods {
                let selected = state.period == *period;
                if ui.selectable_label(selected, *label).clicked() {
                    state.period = *period;
                    state.fetch_stats(ipc_client);
                }
            }
        });

        ui.add_space(12.0);

        if !state.stats_enabled {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("⏸").size(48.0));
                ui.add_space(8.0);
                ui.strong(sd.map_or("Statistics recording is disabled", |s| s.stats_disabled.as_str()));
                ui.add_space(4.0);
                ui.small("Enable statistics to start collecting expansion data.");
                ui.add_space(8.0);
                if ui
                    .button(sd.map_or("⚡ Enable Statistics", |s| s.enable_stats.as_str()))
                    .clicked()
                {
                    state.enable_stats();
                }
            });
            return;
        }

        if state.is_loading {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.spinner();
            });
            return;
        }

        let summary = match &state.summary {
            Some(s) => s,
            None => {
                state.fetch_stats(ipc_client);
                ui.spinner();
                return;
            }
        };

        if summary.total_expansions == 0 {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("📊").size(48.0));
                ui.add_space(8.0);
                ui.strong(sd.map_or("No data yet", |s| s.no_data.as_str()));
                ui.small("Start using espanso to collect expansion statistics.");
            });
            return;
        }

        // Summary cards
        ui.horizontal(|ui| {
            summary_card(
                ui,
                sd.map_or("Total Expansions", |s| s.total_expansions.as_str()),
                &summary.total_expansions.to_string(),
                egui::Color32::from_rgb(51, 102, 255),
            );
            ui.add_space(12.0);
            summary_card(
                ui,
                sd.map_or("Active Matches", |s| s.active_matches.as_str()),
                &summary.active_matches.to_string(),
                egui::Color32::from_rgb(34, 139, 34),
            );
            ui.add_space(12.0);
            summary_card(
                ui,
                sd.map_or("Unused Matches", |s| s.unused_matches.as_str()),
                &summary.unused_matches.to_string(),
                egui::Color32::from_rgb(255, 140, 0),
            );
        });

        ui.add_space(16.0);

        ui.strong(sd.map_or("Top Triggers", |s| s.top_triggers.as_str()));
        if summary.top_triggers.is_empty() {
            ui.small("No data available for this period.");
        } else {
            render_bar_chart(ui, &summary.top_triggers);
        }
    });
}

fn summary_card(ui: &mut egui::Ui, label: &str, value: &str, accent: egui::Color32) {
    let frame = egui::Frame::none()
        .fill(ui.visuals().extreme_bg_color)
        .rounding(6.0)
        .inner_margin(16.0)
        .stroke(egui::Stroke::new(1.0, accent.gamma_multiply(0.3)));

    frame.show(ui, |ui| {
        ui.set_min_width(160.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(value).size(32.0).color(accent).strong());
            ui.small(label);
        });
    });
}

fn render_bar_chart(ui: &mut egui::Ui, triggers: &[TriggerStat]) {
    if triggers.is_empty() {
        return;
    }

    let max_count = triggers.iter().map(|t| t.count).max().unwrap_or(1).max(1);
    let chart_height = 180.0;
    let bar_width = 60.0;
    let bar_spacing = 12.0;
    let available_width = ui.available_width();

    let (_rect, response) = ui.allocate_exact_size(
        egui::vec2(available_width, chart_height + 40.0),
        egui::Sense::hover(),
    );

    let base_y = response.rect.bottom() - 25.0;

    ui.painter().line_segment(
        [
            egui::pos2(response.rect.left(), base_y),
            egui::pos2(response.rect.right(), base_y),
        ],
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
    );

    for (i, trigger) in triggers.iter().enumerate() {
        let bar_height = (trigger.count as f32 / max_count as f32) * chart_height;
        let x = response.rect.left() + i as f32 * (bar_width + bar_spacing) + bar_spacing;

        if x + bar_width > response.rect.right() {
            break;
        }

        let bar_rect = egui::Rect::from_min_size(
            egui::pos2(x, base_y - bar_height),
            egui::vec2(bar_width, bar_height),
        );

        let accent = egui::Color32::from_rgb(51, 102, 255);
        ui.painter().rect_filled(bar_rect, 4.0, accent);

        ui.painter().text(
            egui::pos2(bar_rect.center().x, bar_rect.top() - 6.0),
            egui::Align2::CENTER_BOTTOM,
            trigger.count.to_string(),
            egui::FontId::proportional(11.0),
            ui.visuals().strong_text_color(),
        );

        ui.painter().text(
            egui::pos2(bar_rect.center().x, base_y + 14.0),
            egui::Align2::CENTER_TOP,
            &trigger.trigger,
            egui::FontId::proportional(10.0),
            ui.visuals().weak_text_color(),
        );
    }
}

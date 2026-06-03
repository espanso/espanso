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

//! Module 4: Trigger Tester — test match expansion without leaving the GUI.

use crate::i18n::Translations;
use crate::ipc::IpcClient;

/// Result from a test expansion.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub matched: bool,
    pub match_trigger: Option<String>,
    pub match_file: Option<String>,
    pub match_line: Option<u32>,
    pub match_type: Option<String>,
    pub rendered_output: Option<String>,
    pub elapsed_us: u64,
}

/// State for the trigger tester module.
pub struct TriggerTesterState {
    trigger_input: String,
    app_title: String,
    app_class: String,
    app_exec: String,
    test_result: Option<TestResult>,
    is_testing: bool,
}

impl TriggerTesterState {
    pub fn new() -> Self {
        TriggerTesterState {
            trigger_input: String::new(),
            app_title: String::new(),
            app_class: String::new(),
            app_exec: String::new(),
            test_result: None,
            is_testing: false,
        }
    }
}

/// Render the trigger tester module.
pub fn show(
    ui: &mut egui::Ui,
    state: &mut TriggerTesterState,
    t: &Translations,
    _ipc_client: &IpcClient,
) {
    let tt = t.trigger_tester.as_ref();

    ui.vertical(|ui| {
        ui.heading(tt.map_or("Trigger Tester", |tt| tt.title.as_str()));

        ui.add_space(12.0);

        // === Input area ===
        ui.label(tt.map_or("Type a trigger to test", |tt| tt.input_label.as_str()));
        ui.add(
            egui::TextEdit::singleline(&mut state.trigger_input)
                .hint_text(tt.map_or(":hello", |tt| tt.input_placeholder.as_str()))
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace),
        );

        ui.add_space(8.0);

        // === App context (collapsible) ===
        egui::CollapsingHeader::new(
            tt.map_or("Application context (optional)", |tt| tt.app_context_label.as_str()),
        )
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Title:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.app_title)
                        .hint_text(tt.map_or("Window title...", |tt| tt.app_context_title.as_str())),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Class:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.app_class)
                        .hint_text(tt.map_or("Window class...", |tt| tt.app_context_class.as_str())),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Process:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.app_exec)
                        .hint_text(tt.map_or("Process name...", |tt| tt.app_context_exec.as_str())),
                );
            });
        });

        ui.add_space(12.0);

        // === Test button ===
        let test_btn = egui::Button::new(
            egui::RichText::new(tt.map_or("▶ Test Expansion", |tt| tt.test_button.as_str()))
                .size(16.0),
        )
        .fill(egui::Color32::from_rgb(51, 102, 255))
        .min_size(egui::vec2(200.0, 40.0));

        if ui.add(test_btn).clicked() && !state.trigger_input.is_empty() {
            state.is_testing = true;
            // TODO: Actually run the matching and rendering pipeline
            // For now, simulate a result
            state.test_result = Some(TestResult {
                matched: true,
                match_trigger: Some(state.trigger_input.clone()),
                match_file: Some("base.yml".to_string()),
                match_line: Some(15),
                match_type: Some("Rolling (Trie)".to_string()),
                rendered_output: Some("[rendered output will appear here]".to_string()),
                elapsed_us: 42,
            });
            state.is_testing = false;
        }

        ui.add_space(16.0);

        // === Results area ===
        if let Some(ref result) = state.test_result {
            if result.matched {
                // Match found
                egui::Frame::none()
                    .fill(ui.visuals().extreme_bg_color)
                    .rounding(6.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.strong(tt.map_or("Match Result", |tt| tt.match_result.as_str()));

                        ui.add_space(4.0);

                        if let Some(ref trigger) = result.match_trigger {
                            if let Some(ref file) = result.match_file {
                                let rule_text = tt.map_or(
                                    format!("Matched rule: {} ({}#{})", trigger, file, result.match_line.unwrap_or(0)),
                                    |tt| tt.matched_rule
                                        .replace("{}", trigger)
                                        .replace("{}", file)
                                        .replace("{}", &result.match_line.unwrap_or(0).to_string()),
                                );
                                ui.label(&rule_text);
                            }
                        }

                        if let Some(ref mtype) = result.match_type {
                            let type_text = tt.map_or(
                                format!("Match type: {}", mtype),
                                |tt| tt.match_type.replace("{}", mtype),
                            );
                            ui.label(&type_text);
                        }

                        ui.add_space(8.0);

                        ui.strong(tt.map_or("Rendered output:", |tt| tt.rendered_output.as_str()));
                        if let Some(ref output) = result.rendered_output {
                            ui.add_space(4.0);
                            let mut output_clone = output.clone();
                            ui.add(
                                egui::TextEdit::multiline(&mut output_clone)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(2)
                                    .interactive(false)
                                    .font(egui::TextStyle::Monospace),
                            );
                        }

                        ui.add_space(8.0);
                        let elapsed_text = tt.map_or(
                            format!("Elapsed: {}µs", result.elapsed_us),
                            |tt| tt.elapsed.replace("{}", &format!("{}µs", result.elapsed_us)),
                        );
                        ui.small(&elapsed_text);
                    });
            } else {
                // No match
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("🔍").size(32.0));
                    ui.strong(tt.map_or("No match found for this input", |tt| tt.no_match.as_str()));
                });
            }

            // Script warning
            ui.add_space(8.0);
            ui.small(
                egui::RichText::new(format!(
                    "⚠️ {}",
                    tt.map_or(
                        "Script/shell variables require a running Worker to execute",
                        |tt| tt.script_warning.as_str(),
                    )
                ))
                .color(ui.visuals().warn_fg_color),
            );
        }

        // Loading state
        if state.is_testing {
            ui.add_space(8.0);
            ui.spinner();
        }
    });
}

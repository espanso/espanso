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

use std::path::PathBuf;

use crate::backend::tester::{test_expansion, TestResult};
use crate::i18n::Translations;
use crate::ipc::IpcClient;

pub struct TriggerTesterState {
    trigger_input: String,
    app_title: String,
    app_class: String,
    app_exec: String,
    test_result: Option<TestResult>,
    is_testing: bool,
    config_dir: Option<PathBuf>,
    error_msg: Option<String>,
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
            config_dir: None,
            error_msg: None,
        }
    }

    pub fn set_config_dir(&mut self, dir: Option<PathBuf>) {
        self.config_dir = dir;
    }
}

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

        // Input area
        ui.label(tt.map_or("Type a trigger to test:", |tt| tt.input_label.as_str()));
        let trigger_response = ui.add(
            egui::TextEdit::singleline(&mut state.trigger_input)
                .hint_text(tt.map_or(":hello", |tt| tt.input_placeholder.as_str()))
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace),
        );

        ui.add_space(8.0);

        // App context (collapsible)
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

        // Test button
        let test_btn = egui::Button::new(
            egui::RichText::new(tt.map_or("▶ Test Expansion", |tt| tt.test_button.as_str()))
                .size(16.0),
        )
        .fill(egui::Color32::from_rgb(51, 102, 255))
        .min_size(egui::vec2(200.0, 40.0));

        if ui.add(test_btn).clicked() && !state.trigger_input.is_empty() {
            state.is_testing = true;
            state.test_result = None;
            state.error_msg = None;

            if let Some(ref config_dir) = state.config_dir {
                let result = test_expansion(
                    config_dir,
                    &state.trigger_input,
                    if state.app_title.is_empty() { None } else { Some(state.app_title.as_str()) },
                    if state.app_class.is_empty() { None } else { Some(state.app_class.as_str()) },
                    if state.app_exec.is_empty() { None } else { Some(state.app_exec.as_str()) },
                );
                match result {
                    Ok(r) => state.test_result = Some(r),
                    Err(e) => state.error_msg = Some(format!("Test failed: {}", e)),
                }
            } else {
                state.error_msg = Some("No config directory configured".to_string());
            }
            state.is_testing = false;
        }

        // Also test on Enter press in the trigger input
        if trigger_response.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
            && !state.trigger_input.is_empty()
        {
            state.is_testing = true;
            state.test_result = None;
            state.error_msg = None;

            if let Some(ref config_dir) = state.config_dir {
                let result = test_expansion(
                    config_dir,
                    &state.trigger_input,
                    None, None, None,
                );
                match result {
                    Ok(r) => state.test_result = Some(r),
                    Err(e) => state.error_msg = Some(format!("Test failed: {}", e)),
                }
            }
            state.is_testing = false;
        }

        ui.add_space(16.0);

        // Loading state
        if state.is_testing {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Testing...");
            });
        }

        // Error message
        if let Some(ref err) = state.error_msg {
            ui.colored_label(egui::Color32::from_rgb(255, 107, 107), err);
        }

        // Results area
        if let Some(ref result) = state.test_result {
            if result.matched {
                egui::Frame::none()
                    .fill(ui.visuals().extreme_bg_color)
                    .rounding(6.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.strong(tt.map_or("Match Result", |tt| tt.match_result.as_str()));
                        ui.add_space(4.0);

                        // Match info
                        if let Some(ref trigger) = result.match_trigger {
                            ui.label(format!("Trigger: {}", trigger));
                        }
                        ui.label(format!("Match type: {}", result.match_type));

                        if let Some(ref file) = result.match_file {
                            let line = result.match_line.unwrap_or(0);
                            let rule_text = tt.map_or(
                                format!("Matched rule: {} ({}#{})", result.match_trigger.as_deref().unwrap_or("?"), file, line),
                                |tt| tt.matched_rule
                                    .replace("{}", result.match_trigger.as_deref().unwrap_or("?"))
                                    .replace("{}", file)
                                    .replace("{}", &line.to_string()),
                            );
                            ui.label(&rule_text);
                        }

                        ui.add_space(8.0);

                        // Rendered output
                        ui.strong(tt.map_or("Rendered output:", |tt| tt.rendered_output.as_str()));
                        if let Some(ref output) = result.rendered_output {
                            ui.add_space(4.0);
                            let mut output_display = output.clone();
                            ui.add(
                                egui::TextEdit::multiline(&mut output_display)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(2)
                                    .interactive(false)
                                    .font(egui::TextStyle::Monospace),
                            );
                        }

                        ui.add_space(8.0);

                        // Timing
                        let elapsed_text = tt.map_or(
                            format!("Elapsed: {}µs", result.elapsed_us),
                            |tt| tt.elapsed.replace("{}", &format!("{}µs", result.elapsed_us)),
                        );
                        ui.small(&elapsed_text);
                    });
            } else {
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
    });
}

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

mod app;
mod backend;
mod i18n;
mod ipc;
mod modules;
mod theme;
mod widgets;

use std::path::PathBuf;
use std::env;

use app::{EspansoGuiApp, Module};
use log::info;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .ok();

    // Parse args manually for simplicity (no clap dependency)
    let args: Vec<String> = env::args().collect();

    // Look for --config_dir and --runtime_dir flags
    let mut config_dir: Option<PathBuf> = None;
    let mut runtime_dir: Option<PathBuf> = None;
    let mut initial_module = Module::MatchManager;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config_dir" => {
                if i + 1 < args.len() {
                    config_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            arg if arg.starts_with("--config_dir=") => {
                config_dir = Some(PathBuf::from(&arg[13..]));
            }
            "--runtime_dir" => {
                if i + 1 < args.len() {
                    runtime_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            arg if arg.starts_with("--runtime_dir=") => {
                runtime_dir = Some(PathBuf::from(&arg[14..]));
            }
            arg if !arg.starts_with("--") => {
                initial_module = Module::from_str(arg);
            }
            _ => {}
        }
        i += 1;
    }

    // Default paths
    let config_dir = config_dir.or_else(|| dirs::config_dir().map(|d| d.join("espanso")));
    let runtime_dir = runtime_dir.or_else(|| dirs::cache_dir().map(|d| d.join("espanso")));

    info!("espanso-gui v{} starting...", VERSION);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 680.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "espanso",
        native_options,
        Box::new(|cc| {
            // Set up fonts
            let mut fonts = egui::FontDefinitions::default();
            for font_data in fonts.font_data.values_mut() {
                std::sync::Arc::make_mut(font_data).tweak.scale = 1.05;
            }
            cc.egui_ctx.set_fonts(fonts);

            // Apply system theme
            theme::apply_system_theme(&cc.egui_ctx);

            let app_state = EspansoGuiApp::new(cc, config_dir.clone(), runtime_dir.clone(), initial_module);

            Ok(Box::new(app_state))
        }),
    )
    .expect("Failed to start espanso GUI");
}

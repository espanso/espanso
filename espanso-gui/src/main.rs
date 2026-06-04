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
use std::sync::Arc;

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

    let args: Vec<String> = env::args().collect();

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
            setup_fonts(&cc.egui_ctx);
            theme::apply_system_theme(&cc.egui_ctx);
            let app_state = EspansoGuiApp::new(
                cc,
                config_dir.clone(),
                runtime_dir.clone(),
                initial_module,
            );
            Ok(Box::new(app_state))
        }),
    )
    .expect("Failed to start espanso GUI");
}

/// Load system CJK fonts so Chinese/Japanese/Korean text renders correctly.
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Try to find and load a CJK font from common system locations
    let cjk_paths = find_cjk_font_paths();

    for (family_name, font_path) in &cjk_paths {
        match std::fs::read(font_path) {
            Ok(bytes) => {
                info!("Loaded CJK font '{}' from {}", family_name, font_path);
                fonts.font_data.insert(
                    family_name.clone(),
                    Arc::new(egui::FontData::from_owned(bytes).tweak(
                        egui::FontTweak {
                            scale: 1.0,
                            ..Default::default()
                        },
                    )),
                );
                // Insert this family right after the default proportional family
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push(family_name.clone());
            }
            Err(e) => {
                log::warn!("Could not load CJK font at {}: {}", font_path, e);
            }
        }
    }

    // Slightly larger default font
    for font_data in fonts.font_data.values_mut() {
        std::sync::Arc::make_mut(font_data).tweak.scale = 1.05;
    }

    ctx.set_fonts(fonts);
}

/// Search common system paths for CJK fonts.
/// Returns Vec of (family_name, file_path) for fonts found.
fn find_cjk_font_paths() -> Vec<(String, String)> {
    let mut paths = Vec::new();

    // macOS CJK fonts
    #[cfg(target_os = "macos")]
    {
        let candidates = [
            // PingFang (modern Chinese UI font, preferred)
            "/System/Library/Fonts/PingFang.ttc",
            // STHeiti (Chinese, Japanese-capable)
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            // Songti (Chinese serif)
            "/System/Library/Fonts/Supplemental/Songti.ttc",
            // Apple SD Gothic Neo (Korean-capable)
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            // Hiragino Sans (Japanese)
            "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
            "/System/Library/Fonts/ヒラギノ角ゴシック W4.ttc",
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                // Extract a clean family name from the path
                let name = std::path::Path::new(path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .replace(" Medium", "")
                    .replace(" Light", "")
                    .replace(" W3", "")
                    .replace(" W4", "");
                paths.push((format!("cjk_{}", name), path.to_string()));
            }
        }
    }

    // Windows CJK fonts
    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
        let candidates = [
            format!("{}\\Fonts\\msyh.ttc", windir),   // Microsoft YaHei
            format!("{}\\Fonts\\simsun.ttc", windir),  // SimSun
            format!("{}\\Fonts\\msgothic.ttc", windir), // MS Gothic (Japanese)
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                let name = std::path::Path::new(path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                paths.push((format!("cjk_{}", name), path.clone()));
            }
        }
    }

    // Linux CJK fonts
    #[cfg(target_os = "linux")]
    {
        let candidates = [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                paths.push(("cjk_sans".to_string(), path.to_string()));
                break; // One is enough
            }
        }
    }

    paths
}

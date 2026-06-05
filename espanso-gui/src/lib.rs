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

//! espanso-gui: Management GUI embedded in the espanso worker process.
//!
//! Call [`run`] to open the management panel in its own window & thread.

pub mod app;
pub mod backend;
pub mod command_palette;
pub mod i18n;
pub mod ipc;
pub mod modules;
pub mod theme;
pub mod widgets;

use std::path::PathBuf;
use std::sync::Arc;

use app::{EspansoGuiApp, Module};
use log::info;

/// Run the espanso management GUI in its own window.
///
/// This function blocks until the window closes.
/// Call from a dedicated thread if you don't want to block.
pub fn run(config_dir: Option<PathBuf>, runtime_dir: Option<PathBuf>, initial_module: Module) {
    info!("espanso-gui v{} starting...", env!("CARGO_PKG_VERSION"));

    let config_dir = config_dir.or_else(|| find_default_config_dir());
    let runtime_dir = runtime_dir.or_else(|| dirs::cache_dir().map(|d| d.join("espanso")));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 680.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "文本扩展管理",
        native_options,
        Box::new(move |cc| {
            setup_fonts(&cc.egui_ctx);
            theme::apply_system_theme(&cc.egui_ctx);
            let app_state = EspansoGuiApp::new(cc, config_dir.clone(), runtime_dir.clone(), initial_module);
            Ok(Box::new(app_state))
        }),
    )
    .expect("Failed to start espanso GUI");
}

/// Load system CJK fonts so Chinese/Japanese/Korean text renders correctly.
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let cjk_paths = find_cjk_font_paths();

    for (family_name, font_path) in &cjk_paths {
        match std::fs::read(font_path) {
            Ok(bytes) => {
                info!("Loaded CJK font '{}' from {}", family_name, font_path);
                fonts.font_data.insert(
                    family_name.clone(),
                    Arc::new(egui::FontData::from_owned(bytes).tweak(egui::FontTweak {
                        scale: 1.0,
                        ..Default::default()
                    })),
                );
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

    for font_data in fonts.font_data.values_mut() {
        std::sync::Arc::make_mut(font_data).tweak.scale = 1.05;
    }

    ctx.set_fonts(fonts);
}

/// Search common system paths for CJK fonts.
fn find_cjk_font_paths() -> Vec<(String, String)> {
    let mut paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        let candidates = [
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/Supplemental/Songti.ttc",
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
            "/System/Library/Fonts/ヒラギノ角ゴシック W4.ttc",
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
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

    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
        let candidates = [
            format!("{}\\Fonts\\msyh.ttc", windir),
            format!("{}\\Fonts\\simsun.ttc", windir),
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

    #[cfg(target_os = "linux")]
    {
        let candidates = [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                paths.push(("cjk_sans".to_string(), path.to_string()));
                break;
            }
        }
    }

    paths
}

/// Find the default config directory, matching espanso main's resolution order.
fn find_default_config_dir() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let portable = parent.join(".espanso");
            if portable.is_dir() {
                return Some(portable);
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let dot_espanso = home.join(".espanso");
        if dot_espanso.is_dir() {
            return Some(dot_espanso);
        }
        let config_espanso = home.join(".config").join("espanso");
        if config_espanso.is_dir() {
            return Some(config_espanso);
        }
        #[cfg(target_os = "macos")]
        {
            let legacy = home.join("Library").join("Preferences").join("espanso");
            if legacy.is_dir() {
                return Some(legacy);
            }
        }
    }

    dirs::config_dir().map(|d| d.join("espanso"))
}

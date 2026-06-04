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

//! Real image thumbnails for the espanso GUI.
//!
//! Loads images with the `image` crate, resizes them to a sensible GPU
//! cache size, uploads as egui textures, and renders them with
//! aspect-ratio preservation.  Falls back to an emoji placeholder for
//! missing / unreadable images.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum pixel length on the longest edge for cached GPU textures.
const TEXTURE_SIZE_LIMIT: f32 = 256.0;

/// Maximum number of entries in the global texture cache.
const MAX_CACHE_ENTRIES: usize = 100;

// ---------------------------------------------------------------------------
// Global cache
// ---------------------------------------------------------------------------

/// Per-frame cache for uploaded thumbnail textures.
///
/// A single global static is used so the cache lives across frames without
/// explicit storage in the app state.  The egui runtime is single-threaded,
/// so the `Mutex` is never contended in practice.
static TEXTURE_CACHE: std::sync::LazyLock<Mutex<HashMap<PathBuf, CachedThumbnail>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

struct CachedThumbnail {
    handle: egui::TextureHandle,
    tex_size: [usize; 2],
    modified: Option<std::time::SystemTime>,
    last_used: std::time::Instant,
}

// ---------------------------------------------------------------------------
// Image loading and GPU upload
// ---------------------------------------------------------------------------

/// Load an image from disk, resize to [`TEXTURE_SIZE_LIMIT`] on the
/// longest edge (preserving aspect ratio), upload as an egui texture, and
/// return the texture ID together with the pixel dimensions.
///
/// Returns `None` if the file cannot be opened or decoded.
fn load_and_upload(ctx: &egui::Context, path: &Path) -> Option<(egui::TextureHandle, [usize; 2])> {
    let img = image::io::Reader::open(path).ok()?.decode().ok()?;

    let (w, h) = (img.width() as f32, img.height() as f32);
    let max = w.max(h);

    let rgba = if max > TEXTURE_SIZE_LIMIT {
        let scale = TEXTURE_SIZE_LIMIT / max;
        let nw = (w * scale).round().max(1.0) as u32;
        let nh = (h * scale).round().max(1.0) as u32;
        let resized = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
        resized.to_rgba8()
    } else {
        img.to_rgba8()
    };

    let tex_size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(tex_size, rgba.as_raw());

    let handle = ctx.load_texture(
        path.to_string_lossy(), // debug name
        color_image,
        egui::TextureOptions::LINEAR,
    );

    Some((handle, tex_size))
}

/// Retrieve a texture ID from the global cache, or load + cache it.
///
/// Returns `(TextureId, [width_px, height_px])`.
fn get_or_create_cached(ctx: &egui::Context, path: &Path) -> Option<(egui::TextureId, [usize; 2])> {
    let now = std::time::Instant::now();
    let modified = std::fs::metadata(path).ok().and_then(|m| m.modified().ok());

    let mut cache = TEXTURE_CACHE.lock().ok()?;

    // Fast path -- cached and still up-to-date
    if let Some(entry) = cache.get(path) {
        if entry.modified == modified {
            // Touch and return
            let id = entry.handle.id();
            let size = entry.tex_size;
            drop(cache);
            if let Ok(mut cache) = TEXTURE_CACHE.lock() {
                if let Some(e) = cache.get_mut(path) {
                    e.last_used = now;
                }
            }
            return Some((id, size));
        }
        // Stale entry -- evict and reload below
        cache.remove(path);
    }

    // Cold path -- load from disk
    let (handle, tex_size) = load_and_upload(ctx, path)?;
    let id = handle.id();

    // Evict oldest entries if we exceed the limit
    while cache.len() >= MAX_CACHE_ENTRIES {
        if let Some(oldest) = cache
            .iter()
            .min_by_key(|(_, e)| e.last_used)
            .map(|(k, _)| k.clone())
        {
            cache.remove(&oldest);
        }
    }

    cache.insert(
        path.to_path_buf(),
        CachedThumbnail {
            handle,
            tex_size,
            modified,
            last_used: now,
        },
    );

    Some((id, tex_size))
}

/// Get image file info (dimensions, format name, file size in KB) without
/// uploading to the GPU.
pub fn get_image_info(path: &Path) -> (usize, usize, String, u64) {
    let file_size = std::fs::metadata(path).map(|m| m.len() / 1024).unwrap_or(0);

    let fmt = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("?")
        .to_uppercase();

    match image::image_dimensions(path) {
        Ok((w, h)) => (w as usize, h as usize, fmt, file_size),
        Err(_) => (0, 0, fmt, file_size),
    }
}

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

/// Resolve an image path that may be relative.
///
/// Search order:
///  1. `config_dir/match/images/<path>`
///  2. `config_dir/<path>`
///  3. As-is (relative to CWD)
fn resolve_image_path(image_path: &str, config_dir: Option<&PathBuf>) -> Option<PathBuf> {
    let p = Path::new(image_path);
    if p.is_absolute() {
        if p.exists() {
            return Some(p.to_path_buf());
        }
        return None; // absolute but missing
    }

    if let Some(cd) = config_dir {
        let c1 = cd.join("match").join("images").join(image_path);
        if c1.exists() {
            return Some(c1);
        }
        let c2 = cd.join(image_path);
        if c2.exists() {
            return Some(c2);
        }
    }

    // Fallback: relative to CWD
    if p.exists() {
        return Some(p.to_path_buf());
    }

    None
}

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

/// Compute a rectangle inside `container` that displays a texture of
/// `tex_size` while preserving its aspect ratio (centred).
fn aspect_fit_rect(container: egui::Rect, tex_size: [usize; 2]) -> egui::Rect {
    let (tw, th) = (tex_size[0] as f32, tex_size[1].max(1) as f32);
    let (cw, ch) = (container.width(), container.height());

    let container_aspect = cw / ch.max(0.001);
    let tex_aspect = tw / th;

    let (dw, dh) = if tex_aspect > container_aspect {
        // image is wider than container -- fit width
        (cw, cw / tex_aspect)
    } else {
        // image is taller or same shape -- fit height
        (ch * tex_aspect, ch)
    };

    egui::Rect::from_center_size(container.center(), egui::vec2(dw, dh))
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Show a thumbnail preview for an image.
///
/// Loads the real image (using the `image` crate), resizes it for the GPU
/// cache, uploads as an egui texture, and paints it with aspect-ratio
/// preservation.  Falls back to an emoji placeholder when the image cannot
/// be loaded or no path is provided.
pub fn show_thumbnail(
    ui: &mut egui::Ui,
    image_path: Option<&str>,
    config_dir: Option<&PathBuf>,
    size: egui::Vec2,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if !ui.is_rect_visible(rect) {
        return response;
    }

    // Try to paint a real image thumbnail ---------------------------------
    if let Some(path_str) = image_path {
        if !path_str.is_empty() {
            if let Some(full_path) = resolve_image_path(path_str, config_dir) {
                if let Some((texture_id, tex_size)) = get_or_create_cached(ui.ctx(), &full_path) {
                    let draw_rect = aspect_fit_rect(rect, tex_size);

                    // Background behind the image (visible on mismatch)
                    let bg = if ui.visuals().dark_mode {
                        egui::Color32::from_gray(30)
                    } else {
                        egui::Color32::from_gray(220)
                    };
                    ui.painter().rect_filled(rect, 4.0, bg);

                    // Paint the scaled texture
                    ui.painter().image(
                        texture_id,
                        draw_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );

                    return response;
                }
            }
        }
    }

    // Fallback placeholder -------------------------------------------------
    let color = if ui.visuals().dark_mode {
        egui::Color32::from_gray(50)
    } else {
        egui::Color32::from_gray(230)
    };
    ui.painter().rect_filled(rect, 4.0, color);

    let center = rect.center();
    let emoji = if image_path.is_some() && !image_path.unwrap_or("").is_empty() {
        "🖼️"
    } else {
        "❓"
    };
    ui.painter().text(
        center,
        egui::Align2::CENTER_CENTER,
        emoji,
        egui::FontId::proportional(size.y * 0.4),
        ui.visuals().widgets.inactive.text_color(),
    );

    // Filename overlay at the bottom of the placeholder
    if let Some(path_str) = image_path {
        if !path_str.is_empty() {
            let filename = Path::new(path_str)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| path_str.to_string());

            let galley = ui.painter().layout_no_wrap(
                filename,
                egui::FontId::proportional(10.0),
                ui.visuals().weak_text_color(),
            );
            ui.painter().galley(
                egui::pos2(
                    center.x - galley.size().x / 2.0,
                    rect.bottom() - galley.size().y - 4.0,
                ),
                galley,
                ui.visuals().weak_text_color(),
            );
        }
    }

    response
}

/// Show a larger image preview in a popup window.
pub fn show_image_preview_popup(
    ctx: &egui::Context,
    image_path: &str,
    config_dir: Option<&PathBuf>,
) {
    let full_path = match resolve_image_path(image_path, config_dir) {
        Some(p) => p,
        None => return,
    };

    egui::Window::new("Image Preview")
        .collapsible(true)
        .resizable(true)
        .default_size([400.0, 350.0])
        .show(ctx, |ui| {
            let available = ui.available_size();
            let max_dim = available.x.min(available.y - 40.0).max(100.0);

            let (w, h, format_str, file_size) = get_image_info(&full_path);

            let (disp_w, disp_h) = if w > 0 && h > 0 {
                let ratio = w as f32 / h as f32;
                if ratio > 1.0 {
                    (max_dim, max_dim / ratio)
                } else {
                    (max_dim * ratio, max_dim)
                }
            } else {
                (max_dim, max_dim * 0.75)
            };

            let display_size = egui::vec2(disp_w, disp_h);
            show_thumbnail(ui, Some(image_path), config_dir, display_size);

            ui.add_space(8.0);
            ui.label(format!("{} \u{00d7} {} px", w, h));
            ui.label(format!("{} KB \u{00b7} {}", file_size, format_str));
            ui.label(format!("Path: {}", full_path.display()));
        });
}

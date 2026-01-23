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

use crate::icon::IconPaths;
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::ArgMatches;
use flate2::read::GzDecoder;
use log::error;
use std::ffi::CStr;
use std::io::Read;
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tar::Archive;

// Scope status constants (must match C++ side)
const SCOPE_STATUS_EMPTY: c_int = 0;
const SCOPE_STATUS_PRESENT: c_int = 1;
const SCOPE_STATUS_MISSING: c_int = 2;
const SCOPE_STATUS_INVALID: c_int = 3;

// Global state to pass paths to the C callbacks
static IMPORT_CONTEXT: Mutex<Option<ImportContext>> = Mutex::new(None);

struct ImportContext {
    config_path: PathBuf,
}

extern "C" fn validate_import_data_callback(
    data: *const c_char,
    config_status: *mut c_int,
    matches_status: *mut c_int,
    packages_status: *mut c_int,
) -> c_int {
    // Safety: Set all to invalid initially
    unsafe {
        *config_status = SCOPE_STATUS_INVALID;
        *matches_status = SCOPE_STATUS_INVALID;
        *packages_status = SCOPE_STATUS_INVALID;
    }

    if data.is_null() {
        return 0;
    }

    let data_str = unsafe {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };

    if data_str.is_empty() {
        unsafe {
            *config_status = SCOPE_STATUS_EMPTY;
            *matches_status = SCOPE_STATUS_EMPTY;
            *packages_status = SCOPE_STATUS_EMPTY;
        }
        return 0;
    }

    // Try to validate the archive and detect scopes
    match validate_archive_and_detect_scopes(data_str) {
        Ok((has_config, has_matches, has_packages)) => {
            unsafe {
                *config_status = if has_config { SCOPE_STATUS_PRESENT } else { SCOPE_STATUS_MISSING };
                *matches_status = if has_matches { SCOPE_STATUS_PRESENT } else { SCOPE_STATUS_MISSING };
                *packages_status = if has_packages { SCOPE_STATUS_PRESENT } else { SCOPE_STATUS_MISSING };
            }
            // Valid if at least one scope is present
            if has_config || has_matches || has_packages {
                1
            } else {
                0
            }
        }
        Err(e) => {
            error!("Import validation error: {}", e);
            // Write error to a temp file for debugging
            let _ = std::fs::write("/tmp/espanso_import_debug.txt", format!("Validation error: {}\nInput length: {}\nFirst 100 chars: {}", e, data_str.len(), &data_str[..data_str.len().min(100)]));
            0
        }
    }
}

fn validate_archive_and_detect_scopes(data: &str) -> Result<(bool, bool, bool)> {
    // Remove whitespace from base64 input
    let filtered: String = data.chars().filter(|c| !c.is_whitespace()).collect();

    // Decode base64
    let decoded = STANDARD.decode(&filtered).context("invalid base64 encoding")?;

    // Decompress gzip
    let mut gzip = GzDecoder::new(&decoded[..]);
    let mut tar_data = Vec::new();
    gzip.read_to_end(&mut tar_data).context("invalid gzip data")?;

    // Read tar archive
    let mut archive = Archive::new(&tar_data[..]);

    let mut has_config = false;
    let mut has_matches = false;
    let mut has_packages = false;

    for entry in archive.entries().context("invalid tar archive")? {
        let entry = entry.context("failed to read archive entry")?;
        let path = entry.path().context("failed to read entry path")?;
        let path_str = path.to_string_lossy();

        // Check which scopes are present based on directory prefixes
        if path_str.starts_with("config/") || path_str == "config" {
            has_config = true;
        } else if path_str.starts_with("matches/") || path_str == "matches" {
            has_matches = true;
        } else if path_str.starts_with("packages/") || path_str == "packages" {
            has_packages = true;
        }
    }

    Ok((has_config, has_matches, has_packages))
}

extern "C" fn perform_import_callback(
    data: *const c_char,
    import_config: c_int,
    import_matches: c_int,
    import_packages: c_int,
    clear_config: c_int,
    clear_matches: c_int,
    clear_packages: c_int,
) -> c_int {
    if data.is_null() {
        return 0;
    }

    let data_str = unsafe {
        match CStr::from_ptr(data).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };

    let context = IMPORT_CONTEXT.lock().unwrap();
    let ctx = match context.as_ref() {
        Some(c) => c,
        None => {
            error!("Import context not initialized");
            return 0;
        }
    };

    match perform_import_internal(
        data_str,
        &ctx.config_path,
        import_config != 0,
        import_matches != 0,
        import_packages != 0,
        clear_config != 0,
        clear_matches != 0,
        clear_packages != 0,
    ) {
        Ok(()) => 1,
        Err(e) => {
            error!("Import failed: {}", e);
            0
        }
    }
}

fn perform_import_internal(
    data: &str,
    config_path: &PathBuf,
    import_config: bool,
    import_matches: bool,
    import_packages: bool,
    clear_config: bool,
    clear_matches: bool,
    clear_packages: bool,
) -> Result<()> {
    use std::fs;
    use tar::EntryType;

    // Remove whitespace from base64 input
    let filtered: String = data.chars().filter(|c| !c.is_whitespace()).collect();

    // Decode base64
    let decoded = STANDARD.decode(&filtered).context("invalid base64 encoding")?;

    // Decompress gzip
    let mut gzip = GzDecoder::new(&decoded[..]);
    let mut tar_data = Vec::new();
    gzip.read_to_end(&mut tar_data).context("invalid gzip data")?;

    // Prepare target directories
    let config_dir = config_path.join("config");
    let matches_dir = config_path.join("match");
    let packages_dir = config_path.join("match").join("packages");

    // Clear directories if requested (clearing can happen even without importing)
    if clear_config {
        if config_dir.exists() {
            fs::remove_dir_all(&config_dir).context("failed to clear config directory")?;
        }
        if import_config {
            fs::create_dir_all(&config_dir).context("failed to create config directory")?;
        }
    }

    if clear_matches {
        // Be careful not to delete packages if we're not clearing them
        if matches_dir.exists() {
            if clear_packages || !packages_dir.exists() {
                fs::remove_dir_all(&matches_dir).context("failed to clear matches directory")?;
            } else {
                // Clear matches but preserve packages
                clear_directory_except(&matches_dir, Some(&packages_dir))?;
            }
        }
        if import_matches {
            fs::create_dir_all(&matches_dir).context("failed to create matches directory")?;
        }
    }

    if clear_packages {
        if packages_dir.exists() {
            fs::remove_dir_all(&packages_dir).context("failed to clear packages directory")?;
        }
        if import_packages {
            fs::create_dir_all(&packages_dir).context("failed to create packages directory")?;
        }
    }

    // Read tar archive and extract files
    let mut archive = Archive::new(&tar_data[..]);

    for entry in archive.entries().context("invalid tar archive")? {
        let mut entry = entry.context("failed to read archive entry")?;
        let entry_path = entry.path().context("failed to read entry path")?.to_path_buf();
        let path_str = entry_path.to_string_lossy();

        // Determine which scope this entry belongs to
        let (target_root, should_import) = if path_str.starts_with("config/") || path_str == "config" {
            (Some(&config_dir), import_config)
        } else if path_str.starts_with("matches/") || path_str == "matches" {
            (Some(&matches_dir), import_matches)
        } else if path_str.starts_with("packages/") || path_str == "packages" {
            (Some(&packages_dir), import_packages)
        } else {
            (None, false)
        };

        if !should_import || target_root.is_none() {
            continue;
        }

        let target_root = target_root.unwrap();

        // Strip the scope prefix from the path
        let relative_path = strip_scope_prefix(&entry_path);

        // Skip if this is just the scope root directory
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let target_path = target_root.join(&relative_path);

        // Validate path doesn't escape target directory
        if !target_path.starts_with(target_root) {
            anyhow::bail!("path traversal attempt: {}", entry_path.display());
        }

        match entry.header().entry_type() {
            EntryType::Directory => {
                fs::create_dir_all(&target_path)
                    .with_context(|| format!("failed to create directory: {}", target_path.display()))?;
            }
            EntryType::Regular => {
                // Ensure parent directory exists
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("failed to create parent directory: {}", parent.display()))?;
                }

                let mut file_data = Vec::new();
                entry.read_to_end(&mut file_data)
                    .with_context(|| format!("failed to read archive entry: {}", entry_path.display()))?;

                fs::write(&target_path, &file_data)
                    .with_context(|| format!("failed to write file: {}", target_path.display()))?;
            }
            _ => {
                // Skip other entry types (symlinks, etc.)
            }
        }
    }

    Ok(())
}

fn strip_scope_prefix(path: &Path) -> PathBuf {
    let mut components = path.components();
    // Skip the first component (the scope prefix: config, matches, or packages)
    components.next();
    components.as_path().to_path_buf()
}

fn clear_directory_except(root: &Path, preserve: Option<&Path>) -> Result<()> {
    use std::fs;
    use walkdir::WalkDir;

    if !root.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(root).min_depth(1).contents_first(true) {
        let entry = entry?;
        let path = entry.path();

        if let Some(preserve_path) = preserve {
            if path.starts_with(preserve_path) {
                continue;
            }
        }

        if entry.file_type().is_dir() {
            fs::remove_dir(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

pub fn import_dialog_main(args: &ArgMatches, icon_paths: &IconPaths) -> i32 {
    let config_path = args.value_of("config_path").expect("missing config_path");

    // Set up the import context for the callbacks
    {
        let mut context = IMPORT_CONTEXT.lock().unwrap();
        *context = Some(ImportContext {
            config_path: PathBuf::from(config_path),
        });
    }

    // Show the import dialog
    espanso_modulo::import_dialog::show(
        icon_paths.wizard_icon.as_ref().map(|p| p.to_string_lossy().to_string()).as_deref(),
        validate_import_data_callback,
        perform_import_callback,
    );

    // Clean up the context
    {
        let mut context = IMPORT_CONTEXT.lock().unwrap();
        *context = None;
    }

    0
}

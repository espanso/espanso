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

use std::path::Path;

use anyhow::{Context, Result};

pub fn open_file_with_preferred_editor(
    file_path: &Path,
    editor_path: Option<&str>,
) -> Result<()> {
    let editor = editor_path.and_then(|path| {
        let trimmed = path.trim();
        (!trimmed.is_empty()).then_some(trimmed)
    });

    if is_yaml_file(file_path) {
        if let Some(editor) = editor {
            open_with_editor(editor, file_path)?;
        } else {
            open_with_default_app(file_path)?;
        }
    } else {
        open_with_default_app(file_path)?;
    }

    Ok(())
}

pub fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml"))
}

pub fn open_with_editor(editor: &str, file_path: &Path) -> Result<()> {
    if cfg!(target_os = "windows") {
        std::process::Command::new(editor)
            .arg(file_path)
            .spawn()
            .context("spawn editor")?;
    } else {
        std::process::Command::new("/bin/bash")
            .arg("-c")
            .arg(format!("{} '{}'", editor, file_path.to_string_lossy()))
            .spawn()
            .context("spawn editor")?;
    }

    Ok(())
}

pub fn open_with_default_app(file_path: &Path) -> Result<()> {
    if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(file_path)
            .spawn()
            .context("spawn open")?;
    } else if cfg!(target_os = "windows") {
        let path_string = file_path.to_string_lossy();
        std::process::Command::new("cmd")
            .args(["/C", "start", "", path_string.as_ref()])
            .spawn()
            .context("spawn start")?;
    } else {
        std::process::Command::new("xdg-open")
            .arg(file_path)
            .spawn()
            .context("spawn xdg-open")?;
    }

    Ok(())
}

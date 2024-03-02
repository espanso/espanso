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

use anyhow::Result;
use std::path::PathBuf;
use thiserror::Error;

pub fn is_espanso_in_path() -> bool {
    PathBuf::from("/usr/local/bin/espanso").is_file()
}

pub fn add_espanso_to_path(_: bool) -> Result<()> {
    let target_link_dir = PathBuf::from("/usr/local/bin");
    let exec_path = get_binary_path()?;

    if !target_link_dir.is_dir() {
        return Err(PathError::UsrLocalBinDirDoesNotExist.into());
    }

    let target_link_path = target_link_dir.join("espanso");

    if let Err(error) = std::os::unix::fs::symlink(exec_path, target_link_path) {
        return Err(PathError::SymlinkError(error).into());
    }

    Ok(())
}

pub fn remove_espanso_from_path(_: bool) -> Result<()> {
    let target_link_path = PathBuf::from("/usr/local/bin/espanso");

    if std::fs::symlink_metadata(&target_link_path).is_err() {
        return Err(PathError::SymlinkNotFound.into());
    }

    if let Err(error) = std::fs::remove_file(&target_link_path) {
        return Err(PathError::SymlinkError(error).into());
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum PathError {
    #[error("/usr/local/bin directory doesn't exist")]
    UsrLocalBinDirDoesNotExist,

    #[error("symlink error: `{0}`")]
    SymlinkError(std::io::Error),

    #[error("symlink does not exist, so there is nothing to remove.")]
    SymlinkNotFound,
}

fn get_binary_path() -> Result<PathBuf> {
    // If executed as part of an AppImage, get the app image path instead of
    // the binary itself (which was extracted in a temp directory).
    if let Some(app_image_path) = std::env::var_os("APPIMAGE") {
        return Ok(PathBuf::from(app_image_path));
    }

    Ok(std::env::current_exe()?)
}

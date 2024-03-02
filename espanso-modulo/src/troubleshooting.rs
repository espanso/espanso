/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::path::{Path, PathBuf};

pub use crate::sys::troubleshooting::show;

pub struct TroubleshootingOptions {
    pub window_icon_path: Option<String>,
    pub error_sets: Vec<ErrorSet>,
    pub is_fatal_error: bool,

    pub handlers: TroubleshootingHandlers,
}

pub struct ErrorSet {
    pub file: Option<PathBuf>,
    pub errors: Vec<ErrorRecord>,
}

pub struct ErrorRecord {
    pub level: ErrorLevel,
    pub message: String,
}

pub enum ErrorLevel {
    Error,
    Warning,
}

type OpenFileCallback = dyn Fn(&Path) + Send;

pub struct TroubleshootingHandlers {
    pub dont_show_again_changed: Option<Box<dyn Fn(bool) + Send>>,
    pub open_file: Option<Box<OpenFileCallback>>,
}

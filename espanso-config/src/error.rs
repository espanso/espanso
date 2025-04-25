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

use anyhow::Error;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct NonFatalErrorSet {
    pub file: PathBuf,
    pub errors: Vec<ErrorRecord>,
}

impl NonFatalErrorSet {
    pub fn new(file: &Path, non_fatal_errors: Vec<ErrorRecord>) -> Self {
        Self {
            file: file.to_owned(),
            errors: non_fatal_errors,
        }
    }

    pub fn single_error(file: &Path, error: Error) -> Self {
        Self {
            file: file.to_owned(),
            errors: vec![ErrorRecord::error(error)],
        }
    }
}

#[derive(Debug)]
pub struct ErrorRecord {
    pub level: ErrorLevel,
    pub error: Error,
}

impl ErrorRecord {
    pub fn error(error: Error) -> Self {
        Self {
            level: ErrorLevel::Error,
            error,
        }
    }

    pub fn warn(error: Error) -> Self {
        Self {
            level: ErrorLevel::Warning,
            error,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorLevel {
    Error,
    Warning,
}

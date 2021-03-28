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
use std::{
  fs::{File, OpenOptions},
  path::Path,
};
use std::{
  io::Write,
  sync::{Arc, Mutex},
};

/// This struct can be passed as an output to the logger to "defer" the
/// decision of the output file
#[derive(Clone)]
pub(crate) struct FileProxy {
  output: Arc<Mutex<Option<File>>>,
}

impl FileProxy {
  pub fn new() -> Self {
    Self {
      output: Arc::new(Mutex::new(None)),
    }
  }

  pub fn set_output_file(&self, path: &Path) -> Result<()> {
    let log_file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .append(true)
      .open(path)?;
    let mut lock = self.output.lock().expect("unable to obtain FileProxy lock");
    *lock = Some(log_file);
    Ok(())
  }
}

impl Write for FileProxy {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    match self.output.lock() {
      Ok(lock) => {
        if let Some(mut output) = lock.as_ref() {
          output.write(buf)
        } else {
          Ok(0)
        }
      }
      Err(_) => Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "lock poison error",
      )),
    }
  }

  fn flush(&mut self) -> std::io::Result<()> {
    match self.output.lock() {
      Ok(lock) => {
        if let Some(mut output) = lock.as_ref() {
          output.flush()
        } else {
          Ok(())
        }
      }
      Err(_) => Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "lock poison error",
      )),
    }
  }
}

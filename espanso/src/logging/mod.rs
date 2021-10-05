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
  output: Arc<Mutex<Output>>,
}

enum Output {
  Memory(Vec<u8>),
  File(File),
}

impl FileProxy {
  pub fn new() -> Self {
    Self {
      output: Arc::new(Mutex::new(Output::Memory(Vec::new()))),
    }
  }

  pub fn set_output_file(&self, path: &Path, read_only: bool, create_new: bool) -> Result<()> {
    // Remove previous log, if present
    if create_new && !read_only && path.is_file() {
      std::fs::remove_file(path)?;
    }

    let mut log_file = OpenOptions::new()
      .read(true)
      .write(!read_only)
      .create(true)
      .append(true)
      .open(path)?;
    let mut lock = self.output.lock().expect("unable to obtain FileProxy lock");

    // Transfer the log content that has been buffered into the file
    if let Output::Memory(buffered) = &mut (*lock) {
      log_file.write_all(buffered)?;
      buffered.clear();
    }
    *lock = Output::File(log_file);
    Ok(())
  }
}

impl Write for FileProxy {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    match self.output.lock() {
      Ok(mut lock) => {
        match &mut (*lock) {
          // Write to the memory buffer until a file is ready
          Output::Memory(buffer) => {
            buffer.write(buf)
          }
          Output::File(output) => {
            output.write(buf)
          }
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
      Ok(mut lock) => {
        match &mut (*lock) {
          Output::Memory(buffer) => {
            buffer.flush()
          }
          Output::File(output) => {
            output.flush()
          }
        }
      }
      Err(_) => Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "lock poison error",
      )),
    }
  }
}

#[macro_export]
macro_rules! info_println {
  ($($tts:tt)*) => {
    println!($($tts)*);
    log::info!($($tts)*);
  }
}

#[macro_export]
macro_rules! warn_eprintln {
  ($($tts:tt)*) => {
    eprintln!($($tts)*);
    log::warn!($($tts)*);
  }
}

#[macro_export]
macro_rules! error_eprintln {
  ($($tts:tt)*) => {
    eprintln!($($tts)*);
    log::error!($($tts)*);
  }
}
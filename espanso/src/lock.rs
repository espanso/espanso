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
use fs2::FileExt;
use std::{
  fs::{File, OpenOptions},
  path::Path,
};

pub struct Lock {
  lock_file: File,
}

impl Lock {
  #[allow(dead_code)]
  pub fn release(self) -> Result<()> {
    self.lock_file.unlock()?;
    Ok(())
  }

  fn acquire(runtime_dir: &Path, name: &str) -> Option<Lock> {
    let lock_file_path = runtime_dir.join(format!("{name}.lock"));
    let lock_file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open(&lock_file_path)
      .unwrap_or_else(|_| panic!("unable to create reference to lock file: {lock_file_path:?}"));
    if lock_file.try_lock_exclusive().is_ok() {
      Some(Lock { lock_file })
    } else {
      None
    }
  }
}

impl Drop for Lock {
  fn drop(&mut self) {
    self
      .lock_file
      .unlock()
      .unwrap_or_else(|_| panic!("unable to unlock lock_file: {:?}", self.lock_file));
  }
}

pub fn acquire_daemon_lock(runtime_dir: &Path) -> Option<Lock> {
  Lock::acquire(runtime_dir, "espanso-daemon")
}

pub fn acquire_worker_lock(runtime_dir: &Path) -> Option<Lock> {
  Lock::acquire(runtime_dir, "espanso-worker")
}

pub fn acquire_legacy_lock(runtime_dir: &Path) -> Option<Lock> {
  Lock::acquire(runtime_dir, "espanso")
}

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
    path::{Path, PathBuf},
};

const DAEMON_LOCK_NAME: &str = "espanso-daemon";
const WORKER_LOCK_NAME: &str = "espanso-worker";

fn lock_file_path(runtime_dir: &Path, name: &str) -> PathBuf {
    runtime_dir.join(format!("{name}.lock"))
}

pub struct Lock {
    lock_file: File,
}

impl Lock {
    #[allow(dead_code)]
    pub fn release(self) -> Result<()> {
        fs2::FileExt::unlock(&self.lock_file)?;
        Ok(())
    }

    fn acquire(runtime_dir: &Path, name: &str) -> Option<Lock> {
        let lock_file_path = lock_file_path(runtime_dir, name);
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&lock_file_path)
            .unwrap_or_else(|_| {
                panic!(
                    "unable to create reference to lock file: {}",
                    lock_file_path.display()
                )
            });
        if lock_file.try_lock_exclusive().is_ok() {
            Some(Lock { lock_file })
        } else {
            None
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        fs2::FileExt::unlock(&self.lock_file)
            .unwrap_or_else(|_| panic!("unable to unlock lock_file: {:?}", self.lock_file));
    }
}

pub fn acquire_daemon_lock(runtime_dir: &Path) -> Option<Lock> {
    Lock::acquire(runtime_dir, DAEMON_LOCK_NAME)
}

pub fn acquire_worker_lock(runtime_dir: &Path) -> Option<Lock> {
    Lock::acquire(runtime_dir, WORKER_LOCK_NAME)
}

#[cfg(target_os = "macos")]
pub fn clear_daemon_lock(runtime_dir: &Path) -> Result<()> {
    let lock_file_path = lock_file_path(runtime_dir, DAEMON_LOCK_NAME);
    if lock_file_path.exists() {
        std::fs::remove_file(&lock_file_path)?;
    }
    Ok(())
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn clear_daemon_lock_allows_takeover_while_stale_lock_is_held() {
        let dir = TempDir::new("espansolock").unwrap();
        let runtime = dir.path();

        // An orphaned daemon acquires the lock and keeps holding it.
        let orphan = acquire_daemon_lock(runtime);
        assert!(orphan.is_some());

        // A new daemon can't acquire the lock while the orphan holds it.
        assert!(acquire_daemon_lock(runtime).is_none());

        // Clearing the stale lock file lets the new daemon take over, even
        // though the orphan is still holding its (now unlinked) lock file.
        clear_daemon_lock(runtime).unwrap();
        let takeover = acquire_daemon_lock(runtime);
        assert!(takeover.is_some());

        // Keep the orphan alive until the end so the assertion above proves
        // the inode swap works, not that the orphan simply released.
        drop(orphan);
    }
}

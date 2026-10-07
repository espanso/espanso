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

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

use anyhow::Result;
use crossbeam::{channel::Sender, select};
use log::{error, info, warn};

const WATCHER_NOTIFY_DELAY_MS: u64 = 500;
const WATCHER_DEBOUNCE_DURATION_MS: u64 = 1000;

pub fn initialize_and_spawn(config_dir: &Path, watcher_notify: Sender<()>) -> Result<()> {
    let config_dir = config_dir.to_path_buf();

    let (debounce_tx, debounce_rx) = crossbeam::channel::unbounded();

    std::thread::Builder::new()
        .name("watcher".to_string())
        .spawn(move || {
            watcher_main(&config_dir, debounce_tx);
        })?;

    std::thread::Builder::new()
        .name("watcher-debouncer".to_string())
        .spawn(move || {
            debouncer_main(debounce_rx, &watcher_notify);
        })?;

    Ok(())
}

fn watcher_main(config_dir: &Path, debounce_tx: Sender<()>) {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher: RecommendedWatcher =
        Watcher::new(tx, Duration::from_millis(WATCHER_NOTIFY_DELAY_MS))
            .expect("unable to create file watcher");

    watcher
        .watch(config_dir, RecursiveMode::Recursive)
        .expect("unable to start file watcher");

    info!("watching for changes in path: {}", config_dir.display());

    // notify (FSEvents on macOS) does not follow symlinks below the watched root, so
    // `match` or `config` linked into e.g. a git repository were never reloaded
    // automatically (#2249, #923). Watch the link targets as well.
    for target in symlink_targets(config_dir) {
        match watcher.watch(&target, RecursiveMode::Recursive) {
            Ok(()) => info!("watching for changes in linked path: {}", target.display()),
            Err(err) => warn!("unable to watch linked path {}: {err:?}", target.display()),
        }
    }

    loop {
        let should_reload = match rx.recv() {
            Ok(event) => {
                let path = match event {
                    DebouncedEvent::Create(path) => Some(path),
                    DebouncedEvent::Write(path) => Some(path),
                    DebouncedEvent::Remove(path) => Some(path),
                    DebouncedEvent::Rename(_, path) => Some(path),
                    _ => None,
                };

                if let Some(path) = path {
                    let extension = path
                        .extension()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_ascii_lowercase();

                    if ["yml", "yaml"].iter().any(|ext| ext == &extension) {
                        // Only load non-hidden yml files
                        !is_file_hidden(&path)
                    } else {
                        // If there is no extension, it's probably a folder
                        extension.is_empty()
                    }
                } else {
                    false
                }
            }
            Err(e) => {
                warn!("error while watching files: {e:?}");
                false
            }
        };

        if should_reload {
            if let Err(error) = debounce_tx.send(()) {
                error!("unable to send watcher file changed event to debouncer: {error}");
            }
        }
    }
}

fn debouncer_main(debounce_rx: crossbeam::channel::Receiver<()>, watcher_notify: &Sender<()>) {
    let mut has_received_event = false;

    loop {
        select! {
          recv(debounce_rx) -> _ => {
            has_received_event = true;
          },
          default(Duration::from_millis(WATCHER_DEBOUNCE_DURATION_MS)) => {
            if has_received_event {
              if let Err(error) = watcher_notify.send(()) {
                error!("unable to send watcher file changed event: {error}");
              }
            }

            has_received_event = false;
          },
        }
    }
}

/// Targets of symlinks in the config dir, in `match/` and in `config/` that point outside of it.
fn symlink_targets(config_dir: &Path) -> Vec<PathBuf> {
    let root = std::fs::canonicalize(config_dir).unwrap_or_else(|_| config_dir.to_path_buf());
    let mut candidates = Vec::new();
    for dir in [
        config_dir.to_path_buf(),
        config_dir.join("match"),
        config_dir.join("config"),
    ] {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            candidates.extend(entries.flatten().map(|entry| entry.path()));
        }
    }

    let mut targets: Vec<PathBuf> = Vec::new();
    for path in candidates {
        let is_symlink =
            std::fs::symlink_metadata(&path).is_ok_and(|meta| meta.file_type().is_symlink());
        if !is_symlink {
            continue;
        }
        let Ok(target) = std::fs::canonicalize(&path) else {
            continue;
        };
        if target.starts_with(&root) || targets.iter().any(|known| target.starts_with(known)) {
            continue;
        }
        targets.retain(|known| !known.starts_with(&target));
        targets.push(target);
    }
    targets
}

fn is_file_hidden(path: &Path) -> bool {
    let starts_with_dot = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .starts_with('.');

    starts_with_dot || has_hidden_attribute(path)
}

#[cfg(windows)]
fn has_hidden_attribute(path: &Path) -> bool {
    use std::os::windows::prelude::*;
    let metadata = std::fs::metadata(path);
    if metadata.is_err() {
        return false;
    }
    let attributes = metadata.unwrap().file_attributes();

    (attributes & 0x2) > 0
}

#[cfg(not(windows))]
fn has_hidden_attribute(_: &Path) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn finds_linked_match_and_config_dirs_outside_the_config_dir() {
        let base = tempdir::TempDir::new("espanso-watch").unwrap();
        let repo = base.path().join("repo");
        let config_dir = base.path().join("espanso");
        std::fs::create_dir_all(repo.join("match")).unwrap();
        std::fs::create_dir_all(repo.join("config")).unwrap();
        std::fs::create_dir_all(config_dir.join("inside")).unwrap();
        std::os::unix::fs::symlink(repo.join("match"), config_dir.join("match")).unwrap();
        std::os::unix::fs::symlink(repo.join("config"), config_dir.join("config")).unwrap();
        // a link pointing inside the config dir needs no extra watch
        std::os::unix::fs::symlink(config_dir.join("inside"), config_dir.join("alias")).unwrap();

        let mut targets = symlink_targets(&config_dir);
        targets.sort();
        let mut expected = vec![
            std::fs::canonicalize(repo.join("config")).unwrap(),
            std::fs::canonicalize(repo.join("match")).unwrap(),
        ];
        expected.sort();
        assert_eq!(targets, expected);
    }

    #[test]
    fn no_links_no_extra_watches() {
        let base = tempdir::TempDir::new("espanso-watch").unwrap();
        std::fs::create_dir_all(base.path().join("match")).unwrap();
        assert!(symlink_targets(base.path()).is_empty());
    }
}

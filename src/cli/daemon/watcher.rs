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

use std::{path::Path, time::Duration};

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

  info!("watching for changes in path: {:?}", config_dir);

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
        warn!("error while watching files: {:?}", e);
        false
      }
    };

    if should_reload {
      if let Err(error) = debounce_tx.send(()) {
        error!(
          "unable to send watcher file changed event to debouncer: {}",
          error
        );
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
            error!("unable to send watcher file changed event: {}", error);
          }
        }

        has_received_event = false;
      },
    }
  }
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

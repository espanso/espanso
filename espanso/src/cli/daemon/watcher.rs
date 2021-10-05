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
  path::Path,
  time::{Duration, Instant},
};

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

use anyhow::Result;
use crossbeam::channel::Sender;
use log::{error, info, warn};

const WATCHER_DEBOUNCE_DURATION: u64 = 1;

pub fn initialize_and_spawn(config_dir: &Path, watcher_notify: Sender<()>) -> Result<()> {
  let config_dir = config_dir.to_path_buf();

  std::thread::Builder::new()
    .name("watcher".to_string())
    .spawn(move || {
      watcher_main(&config_dir, &watcher_notify);
    })?;

  Ok(())
}

fn watcher_main(config_dir: &Path, watcher_notify: &Sender<()>) {
  let (tx, rx) = std::sync::mpsc::channel();

  let mut watcher: RecommendedWatcher =
    Watcher::new(tx, Duration::from_secs(WATCHER_DEBOUNCE_DURATION))
      .expect("unable to create file watcher");

  watcher
    .watch(&config_dir, RecursiveMode::Recursive)
    .expect("unable to start file watcher");

  info!("watching for changes in path: {:?}", config_dir);

  let mut last_event_arrival = Instant::now();

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

    // Send only one event, otherwise we could run the risk of useless reloads or even race conditions.
    if should_reload
      && last_event_arrival.elapsed() > std::time::Duration::from_secs(WATCHER_DEBOUNCE_DURATION)
    {
      if let Err(error) = watcher_notify.send(()) {
        error!("unable to send watcher file changed event: {}", error);
      }
    }

    last_event_arrival = Instant::now();
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
fn has_hidden_attribute(path: &Path) -> bool {
  false
}

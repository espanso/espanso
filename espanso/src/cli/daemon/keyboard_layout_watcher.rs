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
use crossbeam::channel::Sender;
use log::{debug, error, warn};

const WATCHER_INTERVAL: u64 = 1000;

pub fn initialize_and_spawn(watcher_notify: Sender<()>) -> Result<()> {
  // On Windows and macOS we don't need to restart espanso when the layout changes
  if !cfg!(target_os = "linux") {
    return Ok(());
  }

  std::thread::Builder::new()
    .name("keyboard_layout_watcher".to_string())
    .spawn(move || {
      watcher_main(&watcher_notify);
    })?;

  Ok(())
}

fn watcher_main(watcher_notify: &Sender<()>) {
  let mut layout = espanso_detect::get_active_layout();

  if layout.is_none() {
    warn!("keyboard layout watcher couldn't determine active layout.")
  }

  loop {
    std::thread::sleep(std::time::Duration::from_millis(WATCHER_INTERVAL));

    let current_layout = espanso_detect::get_active_layout();
    if current_layout != layout {
      debug!(
        "detected keyboard layout change: from '{}' to '{}'",
        layout.as_deref().unwrap_or_default(),
        current_layout.as_deref().unwrap_or_default(),
      );

      if let Err(error) = watcher_notify.send(()) {
        error!("unable to send keyboard layout changed event: {}", error);
      }

      layout = current_layout;
    }
  }
}

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

#[allow(dead_code)]
pub enum SecureInputEvent {
  Disabled,
  Enabled { app_name: String, app_path: String },
}

#[cfg(not(target_os = "macos"))]
pub fn initialize_and_spawn(_secure_input_send: Sender<SecureInputEvent>) -> Result<()> {
  // NOOP on Windows and Linux
  Ok(())
}

#[cfg(target_os = "macos")]
pub fn initialize_and_spawn(secure_input_sender: Sender<SecureInputEvent>) -> Result<()> {
  std::thread::Builder::new()
    .name("secure-input-monitor".to_string())
    .spawn(move || {
      // TODO: pass interval from config parameter
      secure_input_main(
        secure_input_sender,
        std::time::Duration::from_secs(3),
        std::time::Duration::from_secs(1),
      );
    })?;

  Ok(())
}

#[cfg(target_os = "macos")]
fn secure_input_main(
  secure_input_sender: Sender<SecureInputEvent>,
  min_watch_interval: std::time::Duration,
  max_watch_interval: std::time::Duration,
) {
  use log::{error, info};

  info!("monitoring the status of secure input");

  let mut last_secure_input_pid: Option<i64> = None;
  loop {
    let pid = espanso_mac_utils::get_secure_input_pid();

    if let Some(pid) = pid {
      // Some application is currently on `SecureInput`
      let should_notify = if let Some(old_pid) = last_secure_input_pid {
        // We already detected a `SecureInput` app
        #[allow(clippy::needless_bool)]
        if old_pid != pid {
          // The old app is different from the current one, we should take action
          true
        } else {
          // We already notified this application before
          false
        }
      } else {
        // First time we see this `SecureInput` app, we should take action
        true
      };

      if should_notify {
        let secure_input_app = espanso_mac_utils::get_secure_input_application();

        if let Some((app_name, app_path)) = secure_input_app {
          info!("secure input has been acquired, preventing espanso from working correctly. Our guess is that this is caused by '{}', but there are cases in which the detection is unreliable. Full path: {}", app_name, app_path);

          if let Err(error) =
            secure_input_sender.send(SecureInputEvent::Enabled { app_name, app_path })
          {
            error!("unable to send secure input disabled event: {}", error);
          }
        } else {
          error!("detected secure input, but could not figure out which application triggered it");
        }
      }

      last_secure_input_pid = Some(pid);
    } else {
      // No app is currently keeping `SecureInput`

      // If there was an app with `SecureInput`, notify that is now free
      if last_secure_input_pid.is_some() {
        info!("secure input has been disabled");

        if let Err(error) = secure_input_sender.send(SecureInputEvent::Disabled) {
          error!("unable to send secure input disabled event: {}", error);
        }
      }

      last_secure_input_pid = None
    }

    // If an application is currently keeping secure input, refresh the status more often
    if pid.is_some() {
      std::thread::sleep(max_watch_interval);
    } else {
      std::thread::sleep(min_watch_interval);
    }
  }
}

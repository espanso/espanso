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
  path::PathBuf,
  sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
  },
};

use anyhow::{anyhow, bail, Result};
use lazy_static::lazy_static;
use log::{error, warn};
use std::os::windows::process::CommandExt;
use std::process::Command;
use winrt_notification::{IconCrop, Toast};

const ESPANSO_APP_USER_MODEL_ID: &str = "{5E3B6C0F-1A4D-45C4-8872-D8174702101A}";

lazy_static! {
  static ref SEND_CHANNEL: Arc<Mutex<Option<Sender<String>>>> = Arc::new(Mutex::new(None));
}

pub fn initialize_notification_thread(notification_icon_path: PathBuf) -> Result<()> {
  let (sender, receiver) = channel::<String>();

  {
    let mut lock = SEND_CHANNEL
      .lock()
      .map_err(|e| anyhow!("failed to define shared notification sender: {}", e))?;
    *lock = Some(sender);
  }

  std::thread::Builder::new().name("notification-thread".to_string()).spawn(move || {
    // First determine which AppUserModelID we can use
    lazy_static! {
      static ref APP_USER_MODEL_ID: &'static str = if is_espanso_app_user_model_id_set() {
        ESPANSO_APP_USER_MODEL_ID
      } else {
        warn!("unable to find espanso AppUserModelID in the list of registered ones, falling back to Powershell");
        Toast::POWERSHELL_APP_ID
      };
    }

    while let Ok(message) = receiver.recv() {
      if let Err(err) = Toast::new(&APP_USER_MODEL_ID)
        .icon(&notification_icon_path, IconCrop::Square, "Espanso")
        .title("Espanso")
        .text1(&message)
        .sound(None)
        .show()
        .map_err(|e| anyhow!("failed to show notification: {}", e)) {    
          error!("unable to show notification: {}", err); 
      }
    }
  })?;

  Ok(())
}

pub fn show_notification(msg: &str) -> Result<()> {
  let mut lock = SEND_CHANNEL
    .lock()
    .map_err(|e| anyhow!("unable to acquire notification send channel: {}", e))?;
  match &mut *lock {
    Some(sender) => {
      sender.send(msg.to_string())?;
      Ok(())
    }
    None => bail!("notification sender not available"),
  }
}

fn is_espanso_app_user_model_id_set() -> bool {
  match Command::new("powershell")
    .args(["-c", "get-startapps"])
    .creation_flags(0x0800_0000)
    .output()
  {
    Ok(output) => {
      let output_str = String::from_utf8_lossy(&output.stdout);
      // Check if espanso is present
      output_str
        .lines()
        .any(|line| line.contains(ESPANSO_APP_USER_MODEL_ID))
    }
    Err(err) => {
      error!(
        "unable to determine if AppUserModelID was registered: {}",
        err
      );
      false
    }
  }
}

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

use super::super::Middleware;
use crate::event::{Event, EventType};
use log::error;
use std::env;
use std::path::Path;
use std::process::Command;

pub trait ConfigPathProvider {
  fn get_config_path(&self) -> &Path;
}

pub struct ConfigMiddleware<'a> {
  provider: &'a dyn ConfigPathProvider,
}

impl<'a> ConfigMiddleware<'a> {
  pub fn new(provider: &'a dyn ConfigPathProvider) -> Self {
    Self { provider }
  }
}

impl<'a> Middleware for ConfigMiddleware<'a> {
  fn name(&self) -> &'static str {
    "open_config"
  }

  fn next(&self, event: Event, _dispatch: &mut dyn FnMut(Event)) -> Event {
    let config_path = match self.provider.get_config_path().canonicalize() {
      Ok(path) => path,
      Err(err) => {
        error!(
          "unable to canonicalize the config path into the image resolver: {}",
          err
        );
        self.provider.get_config_path().to_owned()
      }
    };
    if let EventType::ShowConfigFolder = event.etype {
      let program: &str;
      if env::consts::OS == "macos" {
        program = "open";
      } else if env::consts::OS == "windows" {
        program = "explorer";
      } else if env::consts::OS == "linux" {
        program = "xdg-open";
      } else {
        panic!("Unsupported OS")
      }
      Command::new(program).arg(config_path).spawn().unwrap();
      return Event::caused_by(event.source_id, EventType::NOOP);
    }
    event
  }
}

// TODO: test

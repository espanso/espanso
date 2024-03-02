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

use std::path::Path;

use log::error;

use super::super::Middleware;
use crate::event::{internal::ImageResolvedEvent, Event, EventType};

pub trait PathProvider {
  fn get_config_path(&self) -> &Path;
}

pub struct ImageResolverMiddleware<'a> {
  provider: &'a dyn PathProvider,
}

impl<'a> ImageResolverMiddleware<'a> {
  pub fn new(provider: &'a dyn PathProvider) -> Self {
    Self { provider }
  }
}

impl<'a> Middleware for ImageResolverMiddleware<'a> {
  fn name(&self) -> &'static str {
    "image_resolve"
  }

  fn next(&self, event: Event, _: &mut dyn FnMut(Event)) -> Event {
    if let EventType::ImageRequested(m_event) = &event.etype {
      // On Windows, we have to replace the forward / with the backslash \ in the path
      let path = if cfg!(target_os = "windows") {
        m_event.image_path.replace('/', "\\")
      } else {
        m_event.image_path.clone()
      };

      let path = if path.contains("$CONFIG") {
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
        path.replace("$CONFIG", &config_path.to_string_lossy())
      } else {
        path
      };

      return Event::caused_by(
        event.source_id,
        EventType::ImageResolved(ImageResolvedEvent { image_path: path }),
      );
    }

    event
  }
}

// TODO: test

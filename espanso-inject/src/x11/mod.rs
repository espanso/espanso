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

use crate::Injector;

use anyhow::{bail, ensure, Result};
use log::{error, warn};

mod default;
mod ffi;
mod xdotool;

pub struct X11ProxyInjector {
  default_injector: Option<default::X11DefaultInjector>,
  xdotool_injector: Option<xdotool::X11XDOToolInjector>,
}

impl X11ProxyInjector {
  pub fn new() -> Result<Self> {
    let default_injector = match default::X11DefaultInjector::new() {
      Ok(injector) => Some(injector),
      Err(err) => {
        error!("X11DefaultInjector could not be initialized: {:?}", err);
        warn!("falling back to xdotool injector");
        None
      }
    };

    let xdotool_injector = match xdotool::X11XDOToolInjector::new() {
      Ok(injector) => Some(injector),
      Err(err) => {
        error!("X11XDOToolInjector could not be initialized: {:?}", err);
        None
      }
    };

    if default_injector.is_none() && xdotool_injector.is_none() {
      bail!("unable to initialize injectors, neither the default or xdotool fallback could be initialized");
    }

    Ok(X11ProxyInjector {
      default_injector,
      xdotool_injector,
    })
  }

  fn get_active_injector(&self, options: &crate::InjectionOptions) -> Result<&dyn Injector> {
    ensure!(
      self.default_injector.is_some() || self.xdotool_injector.is_some(),
      "unable to get active injector, neither default or xdotool fallback are available."
    );

    if options.x11_use_xdotool_fallback {
      if let Some(xdotool_injector) = self.xdotool_injector.as_ref() {
        return Ok(xdotool_injector);
      } else if let Some(default_injector) = self.default_injector.as_ref() {
        return Ok(default_injector);
      }
    } else if let Some(default_injector) = self.default_injector.as_ref() {
      return Ok(default_injector);
    } else if let Some(xdotool_injector) = self.xdotool_injector.as_ref() {
      return Ok(xdotool_injector);
    }

    unreachable!()
  }
}

impl Injector for X11ProxyInjector {
  fn send_string(&self, string: &str, options: crate::InjectionOptions) -> Result<()> {
    self
      .get_active_injector(&options)?
      .send_string(string, options)
  }

  fn send_keys(&self, keys: &[crate::keys::Key], options: crate::InjectionOptions) -> Result<()> {
    self.get_active_injector(&options)?.send_keys(keys, options)
  }

  fn send_key_combination(
    &self,
    keys: &[crate::keys::Key],
    options: crate::InjectionOptions,
  ) -> Result<()> {
    self
      .get_active_injector(&options)?
      .send_key_combination(keys, options)
  }
}

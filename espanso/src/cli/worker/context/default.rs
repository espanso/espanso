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

use espanso_info::AppInfoProvider;

use super::*;
use crate::cli::worker::config::ConfigManager;

pub struct DefaultContext<'a> {
  config_manager: &'a ConfigManager<'a>,
  app_info_provider: &'a dyn AppInfoProvider,
}

impl<'a> DefaultContext<'a> {
  pub fn new(
    config_manager: &'a ConfigManager<'a>,
    app_info_provider: &'a dyn AppInfoProvider,
  ) -> Self {
    Self {
      config_manager,
      app_info_provider,
    }
  }
}

impl<'a> Context for DefaultContext<'a> {}

impl<'a> ConfigContext for DefaultContext<'a> {
  // fn get_default_config(&self) -> Arc<dyn Config> {
  //   self.config_manager.default()
  // }

  fn get_active_config(&self) -> Arc<dyn Config> {
    self.config_manager.active()
  }
}

impl<'a> AppInfoContext for DefaultContext<'a> {
  fn get_active_app_info(&self) -> AppInfo {
    self.app_info_provider.get_info()
  }
}

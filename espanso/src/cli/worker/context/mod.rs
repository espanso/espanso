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

use std::sync::Arc;

use espanso_config::config::Config;

mod default;
pub use default::DefaultContext;
use espanso_info::AppInfo;

pub trait Context: ConfigContext + AppInfoContext {}

pub trait ConfigContext {
  // fn get_default_config(&self) -> Arc<dyn Config>;
  fn get_active_config(&self) -> Arc<dyn Config>;
}

pub trait AppInfoContext {
  fn get_active_app_info(&self) -> AppInfo;
}

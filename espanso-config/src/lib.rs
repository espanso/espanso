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
#[macro_use]
extern crate lazy_static;

mod util;
mod config;
mod matches;
mod counter;

use std::path::Path;
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use config::Config;


pub struct ConfigSet {

}

impl ConfigSet {
  //fn active(&self, app: AppProperties) -> &'a Config {
    // TODO: using the app properties, check if any of the sub configs match or not. If not, return the default
    // Here a RegexSet might be very useful to efficiently match them.
  //}

  //fn default(&self) -> &'a Config {}
}

pub struct AppProperties<'a> {
  pub title: Option<&'a str>,
  pub class: Option<&'a str>,
  pub exec: Option<&'a str>,
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn todo() {
    
  }
}
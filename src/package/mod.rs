/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

pub(crate) mod default;
use serde::{Serialize, Deserialize};
use std::error::Error;

pub trait PackageManager {
    fn update_index(&mut self, force: bool) -> Result<UpdateResult, Box<dyn Error>>;
    fn is_index_outdated(&self) -> bool;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    name: String,
    title: String,
    version: String,
    repo: String,
    desc: String,
    author: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageIndex {
    #[serde(rename = "lastUpdate")]
    last_update: u64,

    packages: Vec<Package>
}


#[derive(Clone, Debug, PartialEq)]
pub enum UpdateResult {
    NotOutdated,
    Updated,
}
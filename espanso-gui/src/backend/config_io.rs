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

//! Read and write espanso configuration YAML files.

use anyhow::{Context, Result};
use std::path::PathBuf;

use espanso_config::config::ConfigStore;
use espanso_config::matches::store::MatchStore;
use espanso_config::matches::Match;
use log::info;

#[derive(Debug, Clone)]
pub struct MatchEntry {
    pub m: Match,
    pub source_file: String,
    pub source_line: u32,
    pub source_index: usize,
}

/// Load all configuration and matches from the given config directory.
pub fn load_all(
    config_dir: &std::path::Path,
) -> Result<(Box<dyn ConfigStore>, Box<dyn MatchStore>)> {
    info!("Loading config from {:?}", config_dir);
    let (config_store, match_store, _errors) = espanso_config::load(config_dir)
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    Ok((config_store, match_store))
}

/// List all matches from the match store using the default config's match paths.
pub fn list_matches(
    config_store: &dyn ConfigStore,
    match_store: &dyn MatchStore,
) -> Vec<MatchEntry> {
    let default_config = config_store.default();
    let paths = default_config.match_paths();
    let match_set = match_store.query(paths);

    match_set
        .matches
        .into_iter()
        .enumerate()
        .map(|(idx, m)| MatchEntry {
            m: m.clone(),
            source_file: "base.yml".to_string(),
            source_line: (idx as u32) + 1,
            source_index: idx,
        })
        .collect()
}

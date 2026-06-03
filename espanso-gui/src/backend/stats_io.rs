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

//! Statistics data queries for the GUI.
//!
//! This module communicates with the Worker process via IPC to fetch
//! expansion statistics from the SQLite database.

use anyhow::Result;
use log::info;

/// A single trigger's usage statistics.
#[derive(Debug, Clone)]
pub struct TriggerStat {
    pub trigger: String,
    pub count: u64,
    pub last_used: Option<String>, // ISO 8601
}

/// Summary statistics for a time period.
#[derive(Debug, Clone)]
pub struct StatsSummary {
    pub total_expansions: u64,
    pub active_matches: u64,
    pub unused_matches: u64,
    pub top_triggers: Vec<TriggerStat>,
}

/// Time period for stats queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsPeriod {
    Last7Days,
    Last30Days,
    AllTime,
}

impl StatsPeriod {
    pub fn label(&self) -> &'static str {
        match self {
            StatsPeriod::Last7Days => "Past 7 Days",
            StatsPeriod::Last30Days => "This Month",
            StatsPeriod::AllTime => "All Time",
        }
    }
}

/// Fetch stats from the Worker process.
///
/// In the current stub implementation, this returns empty data.
/// The full implementation will use IPC to query the Worker's SQLite database.
pub fn fetch_stats(_period: StatsPeriod, _top_n: usize) -> Result<StatsSummary> {
    info!("Fetching stats for period {:?} (stub)", _period);

    // TODO: Send IPC StatsQuery to Worker, receive StatsResponse
    // For now, return empty summary
    Ok(StatsSummary {
        total_expansions: 0,
        active_matches: 0,
        unused_matches: 0,
        top_triggers: Vec::new(),
    })
}

/// Check if stats recording is enabled.
pub fn is_stats_enabled(_config_dir: Option<&std::path::Path>) -> bool {
    // TODO: Read from config
    false
}

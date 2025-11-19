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

use anyhow::Result;
use crossbeam::channel::{unbounded, Sender};
use log::error;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread;

use super::{CliModule, CliModuleArgs};

static RECORD_SENDER: OnceLock<Sender<espanso_engine::process::StatsRecord>> = OnceLock::new();

pub fn new() -> CliModule {
    CliModule {
        requires_paths: true,
        subcommand: "stats".to_string(),
        entry: stats_main,
        ..Default::default()
    }
}

fn stats_main(args: CliModuleArgs) -> i32 {
    let cli_args = args.cli_args.expect("missing cli_args");
    let paths = args.paths.expect("missing paths for stats");

    if let Some(_t) = cli_args.subcommand_matches("clear") {
        match clear_db_in(&paths.config) {
            Ok(()) => {
                println!("Database cleared.");
                return 0;
            }
            Err(e) => {
                eprintln!("Error clearing DB: {}", e);
                return 1;
            }
        }
    }
    if let Some(t) = cli_args.subcommand_matches("prune") {
        let days: i64 = t
            .value_of("days")
            .and_then(|s| s.parse().ok())
            .unwrap_or(180);
        if days <= 0 {
            eprintln!("Error: days must be positive");
            return 1;
        }
        match prune_db_in(&paths.config, days) {
            Ok(()) => {
                println!("Pruned records older than {} days.", days);
                return 0;
            }
            Err(e) => {
                eprintln!("Error pruning DB: {}", e);
                return 1;
            }
        }
    }

    if let Err(err) = stats_main_impl(&cli_args, &paths.config) {
        eprintln!("Error: {}", err);
        return 1;
    }
    0
}

fn stats_main_impl(matches: &clap::ArgMatches, base_dir: &Path) -> Result<()> {
    let db_path = get_stats_db_path_in(base_dir)?;
    if !db_path.exists() {
        println!("No statistics recorded yet.");
        return Ok(());
    }

    let conn = Connection::open(&db_path)?;
    let period = matches.value_of("period").unwrap_or("all");
    let count: usize = matches
        .value_of("count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let grep = matches.value_of("grep");
    let as_json = matches.is_present("json");

    if as_json {
        display_stats_json(&conn, period, count, grep)?;
    } else {
        display_stats(&conn, period, count, grep)?;
    }
    Ok(())
}

fn display_stats(conn: &Connection, period: &str, count: usize, grep: Option<&str>) -> Result<()> {
    let time_filter = time_filter(period);
    let (total, unique) = query_totals(conn, time_filter, grep)?;

    println!("\nMost used triggers ({}):", period);
    println!("┌─────────────────────┬───────┐");
    println!("│ Trigger             │ Count │");
    println!("├─────────────────────┼───────┤");

    for (trigger, cnt) in query_top(conn, time_filter, grep, count)? {
        let display_trigger: String = trigger.chars().take(19).collect();
        println!("│ {:<19} │ {:>5} │", display_trigger, cnt);
    }

    println!("└─────────────────────┴───────┘");
    println!("\nTotal: {} expansions", total);
    println!("Unique: {} triggers", unique);
    Ok(())
}

fn display_stats_json(
    conn: &Connection,
    period: &str,
    count: usize,
    grep: Option<&str>,
) -> Result<()> {
    let time_filter = time_filter(period);
    let (total, unique) = query_totals(conn, time_filter, grep)?;
    let top = query_top(conn, time_filter, grep, count)?;

    let items: Vec<serde_json::Value> = top
        .into_iter()
        .map(|(trigger, cnt)| serde_json::json!({"trigger": trigger, "count": cnt}))
        .collect();

    let result = serde_json::json!({
        "total": total,
        "unique": unique,
        "top": items,
    });
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn time_filter(period: &str) -> &'static str {
    match period {
        "today" => "AND timestamp >= datetime('now', 'start of day')",
        "week" => "AND timestamp >= datetime('now', '-7 days')",
        "month" => "AND timestamp >= datetime('now', '-30 days')",
        "year" => "AND timestamp >= datetime('now', '-365 days')",
        _ => "",
    }
}

fn query_totals(conn: &Connection, time_filter: &str, grep: Option<&str>) -> Result<(i64, i64)> {
    // Total expansions, respecting grep if present
    let mut total_sql = format!(
        "SELECT COUNT(*) FROM expansions e JOIN triggers t ON e.trigger_id = t.id WHERE 1=1 {}",
        time_filter
    );
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(pattern) = grep {
        total_sql.push_str(" AND t.name LIKE ?");
        params_vec.push(Box::new(pattern.to_string()));
    }
    let total: i64 = conn.query_row(
        &total_sql,
        rusqlite::params_from_iter(params_vec.iter().map(|b| &**b)),
        |row| row.get(0),
    )?;

    // Unique triggers
    let mut unique_sql = format!(
        "SELECT COUNT(*) FROM (SELECT DISTINCT t.id FROM expansions e JOIN triggers t ON e.trigger_id = t.id WHERE 1=1 {}",
        time_filter
    );
    let mut unique_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(pattern) = grep {
        unique_sql.push_str(" AND t.name LIKE ?");
        unique_params.push(Box::new(pattern.to_string()));
    }
    unique_sql.push(')');
    let unique: i64 = conn.query_row(
        &unique_sql,
        rusqlite::params_from_iter(unique_params.iter().map(|b| &**b)),
        |row| row.get(0),
    )?;

    Ok((total, unique))
}

fn query_top(
    conn: &Connection,
    time_filter: &str,
    grep: Option<&str>,
    count: usize,
) -> Result<Vec<(String, i64)>> {
    let mut list_sql = format!(
        "SELECT t.name as trigger, COUNT(*) as count FROM expansions e JOIN triggers t ON e.trigger_id = t.id WHERE 1=1 {}",
        time_filter
    );
    let mut params_any: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(pattern) = grep {
        list_sql.push_str(" AND t.name LIKE ?");
        params_any.push(Box::new(pattern.to_string()));
    }
    list_sql.push_str(" GROUP BY t.name ORDER BY count DESC LIMIT ?");
    let limit: i64 = count as i64;
    params_any.push(Box::new(limit));

    let mut stmt = conn.prepare(&list_sql)?;
    let mut rows = stmt.query(rusqlite::params_from_iter(params_any.iter().map(|b| &**b)))?;
    let mut res = Vec::new();
    while let Some(row) = rows.next()? {
        let trigger: String = row.get(0)?;
        let cnt: i64 = row.get(1)?;
        res.push((trigger, cnt));
    }
    Ok(res)
}

fn get_stats_db_path_in(base_dir: &Path) -> Result<PathBuf> {
    Ok(base_dir.join("stats.db"))
}

pub fn init_stats_db_in(base_dir: &Path) -> Result<()> {
    let db_path = get_stats_db_path_in(base_dir)?;

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&db_path)?;
    let _ = conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;");
    // Create normalized schema (development: if you had a legacy table, run `espanso stats clear` once)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS triggers (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS expansions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            trigger_id INTEGER NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY(trigger_id) REFERENCES triggers(id)
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_expansions_timestamp ON expansions(timestamp)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_expansions_trigger ON expansions(trigger_id)",
        [],
    )?;

    // Background writer thread
    let (sender, receiver) = unbounded::<espanso_engine::process::StatsRecord>();
    if RECORD_SENDER.set(sender).is_err() {
        error!("stats: RECORD_SENDER was already initialized; possible orphaned background thread");
        return Err(anyhow::anyhow!("RECORD_SENDER was already initialized"));
    }
    let db_path_clone = db_path.clone();
    thread::Builder::new()
        .name("espanso-stats-writer".to_string())
        .spawn(move || {
            if let Ok(conn) = Connection::open(&db_path_clone) {
                let _ = conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;");
                let (mut upsert_trigger, mut select_trigger, mut insert_expansion) = match (
                    conn.prepare("INSERT OR IGNORE INTO triggers(name) VALUES (?1)"),
                    conn.prepare("SELECT id FROM triggers WHERE name = ?1"),
                    conn.prepare("INSERT INTO expansions(trigger_id) VALUES (?1)"),
                ) {
                    (Ok(u), Ok(s), Ok(i)) => (u, s, i),
                    _ => {
                        error!("stats: failed to prepare statements; writer not active");
                        return;
                    }
                };

                for rec in receiver {
                    let trigger = rec.trigger; // move once and reuse locally
                    if let Err(e) = upsert_trigger.execute(params![trigger.as_str()]) {
                        error!("stats: failed to insert trigger: {}", e);
                        continue;
                    }
                    let trig_id = select_trigger
                        .query_row(params![trigger.as_str()], |row| row.get::<_, i64>(0))
                        .optional()
                        .ok()
                        .flatten();
                    if let Some(id) = trig_id {
                        if let Err(e) = insert_expansion.execute(params![id]) {
                            error!("stats: failed to insert expansion: {}", e);
                        }
                    }
                }
            } else {
                error!("stats: failed to open database in writer thread");
            }
        })?;
    Ok(())
}

pub fn record_stats(record: espanso_engine::process::StatsRecord) -> Result<()> {
    if let Some(sender) = RECORD_SENDER.get() {
        let _ = sender.send(record);
    }
    Ok(())
}

fn clear_db_in(base_dir: &Path) -> Result<()> {
    let db_path = get_stats_db_path_in(base_dir)?;
    if !db_path.exists() {
        return Ok(());
    }
    let conn = Connection::open(db_path)?;
    conn.execute("DELETE FROM expansions", [])?;
    conn.execute("DELETE FROM triggers", [])?;
    // Optional: reclaim space
    let _ = conn.execute_batch("VACUUM; PRAGMA wal_checkpoint(TRUNCATE);");
    Ok(())
}

fn prune_db_in(base_dir: &Path, days: i64) -> Result<()> {
    let db_path = get_stats_db_path_in(base_dir)?;
    if !db_path.exists() {
        return Ok(());
    }
    let conn = Connection::open(db_path)?;
    // Interpolate the window directly into the SQL statement, as SQLite does not accept bound parameters for modifiers
    let sql = format!(
        "DELETE FROM expansions WHERE timestamp < datetime('now', '-{} days')",
        days
    );
    conn.execute(&sql, [])?;
    Ok(())
}

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
use clap::ArgMatches;
use espanso_config::{
  config::{AppProperties, ConfigStore},
  matches::{store::MatchStore, Match, MatchCause},
};
use serde::Serialize;

pub fn list_main(
  cli_args: &ArgMatches,
  config_store: Box<dyn ConfigStore>,
  match_store: Box<dyn MatchStore>,
) -> Result<()> {
  let only_triggers = cli_args.is_present("onlytriggers");
  let preserve_newlines = cli_args.is_present("preservenewlines");

  let class = cli_args.value_of("class");
  let title = cli_args.value_of("title");
  let exec = cli_args.value_of("exec");

  let config = config_store.active(&AppProperties { title, class, exec });
  let match_set = match_store.query(config.match_paths());

  if cli_args.is_present("json") {
    print_matches_as_json(&match_set.matches)?;
  } else {
    print_matches_as_plain(&match_set.matches, only_triggers, preserve_newlines)?;
  }

  Ok(())
}

pub fn print_matches_as_plain(
  match_list: &[&Match],
  only_triggers: bool,
  preserve_newlines: bool,
) -> Result<()> {
  for m in match_list {
    let triggers = match &m.cause {
      MatchCause::None => vec!["(none)".to_string()],
      MatchCause::Trigger(trigger_cause) => trigger_cause.triggers.clone(),
      MatchCause::Regex(regex_cause) => vec![regex_cause.regex.clone()],
    };

    for trigger in triggers {
      if only_triggers {
        println!("{trigger}");
      } else {
        let description = m.description();
        if let Some(label) = &m.label {
          if preserve_newlines {
            println!("{trigger} - {description} - {label}");
          } else {
            println!(
              "{} - {} - {}",
              trigger,
              description.replace('\n', " "),
              label
            );
          }
        } else if preserve_newlines {
          println!("{trigger} - {description}");
        } else {
          println!("{} - {}", trigger, description.replace('\n', " "));
        }
      }
    }
  }

  Ok(())
}

#[derive(Debug, Serialize)]
struct JsonMatchEntry {
  triggers: Vec<String>,
  replace: String,
  label: Option<String>,
}

pub fn print_matches_as_json(match_list: &[&Match]) -> Result<()> {
  let mut entries = Vec::new();
  for m in match_list {
    let triggers = match &m.cause {
      MatchCause::None => vec!["(none)".to_string()],
      MatchCause::Trigger(trigger_cause) => trigger_cause.triggers.clone(),
      MatchCause::Regex(regex_cause) => vec![regex_cause.regex.clone()],
    };

    entries.push(JsonMatchEntry {
      triggers,
      replace: m.description().to_string(),
      label: m.label.clone(),
    });
  }

  let json = serde_json::to_string_pretty(&entries)?;

  println!("{json}");

  Ok(())
}

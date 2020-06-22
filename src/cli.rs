/*
 * This file is part of espanso.
 *
 * Copyright (C) 2020 Federico Terzi
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

use serde::Serialize;
use crate::config::ConfigSet;
use crate::matcher::{Match, MatchContentType};

pub fn list_matches(config_set: ConfigSet, onlytriggers: bool, preserve_newlines: bool) {
    let matches = filter_matches(config_set);

    for m in matches {
        for trigger in m.triggers.iter() {
            if onlytriggers {
                println!("{}", trigger);
            }else {
                match m.content {
                    MatchContentType::Text(ref text) => {
                        let replace = if preserve_newlines {
                            text.replace.to_owned()
                        }else{
                            text.replace.replace("\n", " ")
                        };
                        println!("{} - {}", trigger, replace)
                    },
                    MatchContentType::Image(_) => {
                        // Skip image matches for now
                    },
                }
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonMatchEntry {
    triggers: Vec<String>,
    replace: String,
}

pub fn list_matches_as_json(config_set: ConfigSet) {
    let matches = filter_matches(config_set);

    let mut entries = Vec::new();

    for m in matches {
        match m.content {
            MatchContentType::Text(ref text) => {
                entries.push(JsonMatchEntry {
                    triggers: m.triggers,
                    replace: text.replace.clone(),
                })
            },
            MatchContentType::Image(_) => {
                // Skip image matches for now
            },
        }
    }

    let output = serde_json::to_string(&entries);

    println!("{}", output.unwrap_or_default())
}

fn filter_matches(config_set: ConfigSet) -> Vec<Match> {
    let mut output = Vec::new();
    output.extend(config_set.default.matches);

    // TODO: consider specific matches by class, title or exe path
//    for specific in config_set.specific {
//        output.extend(specific.matches)
//    }
    output
}
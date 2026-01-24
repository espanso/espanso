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
    matches::{
        store::{MatchInfo, MatchStore},
        Match, MatchCause, MatchEffect, TextEffect, TextFormat, TextInjectMode, TriggerCause,
        UpperCasingStyle, Variable,
    },
};
use serde::Serialize;

pub fn explain_main(
    cli_args: &ArgMatches,
    config_store: Box<dyn ConfigStore>,
    match_store: Box<dyn MatchStore>,
) -> Result<()> {
    let trigger = cli_args
        .value_of("trigger")
        .expect("trigger argument is required");
    let show_all = cli_args.is_present("all");
    let json_output = cli_args.is_present("json");

    // Get optional context filters
    let class = cli_args.value_of("class");
    let title = cli_args.value_of("title");
    let exec = cli_args.value_of("exec");

    // Get active config based on app properties (if provided)
    let config = config_store.active(&AppProperties { title, class, exec });

    // Query all matches with source file information
    let matches_with_sources = match_store.query_with_sources(config.match_paths());

    // Find all matches that have this trigger
    let mut candidates: Vec<MatchCandidate> = Vec::new();

    for info in &matches_with_sources {
        if match_has_trigger(info.m, trigger) {
            candidates.push(MatchCandidate {
                info,
                is_selected: false,
            });
        }
    }

    if candidates.is_empty() {
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&ExplainOutputJson {
                    trigger: trigger.to_string(),
                    found: false,
                    selected: None,
                    candidates: vec![],
                })?
            );
        } else {
            println!("Trigger: \"{}\"", trigger);
            println!();
            println!("No matches found for this trigger.");
        }
        return Ok(());
    }

    // The first match in the list is the "selected" one (matches the espanso resolution order)
    candidates[0].is_selected = true;

    if json_output {
        print_json_output(trigger, &candidates, show_all)?;
    } else {
        print_human_output(trigger, &candidates, show_all);
    }

    Ok(())
}

fn match_has_trigger(m: &Match, trigger: &str) -> bool {
    match &m.cause {
        MatchCause::Trigger(cause) => cause.triggers.iter().any(|t| t == trigger),
        MatchCause::Regex(cause) => {
            // For regex matches, check if the trigger is the exact regex pattern
            // (users can search by regex pattern to see regex match details)
            cause.regex == trigger
        }
        MatchCause::None => false,
    }
}

struct MatchCandidate<'a> {
    info: &'a MatchInfo<'a>,
    is_selected: bool,
}

fn print_human_output(trigger: &str, candidates: &[MatchCandidate], show_all: bool) {
    println!("Trigger: \"{}\"", trigger);
    println!();

    // Print the selected match
    if let Some(selected) = candidates.iter().find(|c| c.is_selected) {
        println!("Selected match:");
        print_match_details(selected.info, "  ");
    }

    // Print other candidates if --all flag is set
    if show_all && candidates.len() > 1 {
        println!();
        println!("Other candidates (not selected):");
        for candidate in candidates.iter().filter(|c| !c.is_selected) {
            println!();
            println!("  File: {}", candidate.info.source_file);
            print_match_details(candidate.info, "    ");
            println!("    Reason not selected: lower priority (appears later in resolution order)");
        }
    } else if candidates.len() > 1 {
        println!();
        println!(
            "Note: {} other candidate(s) found. Use --all to see them.",
            candidates.len() - 1
        );
    }
}

fn print_match_details(info: &MatchInfo, indent: &str) {
    let m = info.m;

    println!("{}Defined in: {}", indent, info.source_file);
    println!("{}Match ID: {}", indent, m.id);

    if let Some(label) = &m.label {
        println!("{}Label: {}", indent, label);
    }

    println!("{}Enabled: {}", indent, m.enabled);

    // Print cause details
    match &m.cause {
        MatchCause::Trigger(cause) => {
            println!("{}Type: trigger", indent);
            print_trigger_details(cause, indent);
        }
        MatchCause::Regex(cause) => {
            println!("{}Type: regex", indent);
            println!("{}Regex: {}", indent, cause.regex);
        }
        MatchCause::None => {
            println!("{}Type: none", indent);
        }
    }

    // Print effect details
    match &m.effect {
        MatchEffect::Text(effect) => {
            println!("{}Effect: text replacement", indent);
            print_text_effect_details(effect, indent);
        }
        MatchEffect::Image(effect) => {
            println!("{}Effect: image", indent);
            println!("{}Image path: {}", indent, effect.path);
        }
        MatchEffect::None => {
            println!("{}Effect: none", indent);
        }
    }
}

fn print_trigger_details(cause: &TriggerCause, indent: &str) {
    if cause.triggers.len() == 1 {
        println!("{}Trigger: {}", indent, cause.triggers[0]);
    } else {
        println!("{}Triggers: {:?}", indent, cause.triggers);
    }

    if cause.left_word || cause.right_word {
        let word_mode = match (cause.left_word, cause.right_word) {
            (true, true) => "both",
            (true, false) => "left",
            (false, true) => "right",
            _ => "none",
        };
        println!("{}Word boundary: {}", indent, word_mode);
    }

    if cause.propagate_case {
        let style = match cause.uppercase_style {
            UpperCasingStyle::Uppercase => "uppercase",
            UpperCasingStyle::Capitalize => "capitalize",
            UpperCasingStyle::CapitalizeWords => "capitalize_words",
        };
        println!("{}Propagate case: yes (style: {})", indent, style);
    }
}

fn print_text_effect_details(effect: &TextEffect, indent: &str) {
    // Truncate long replacements for display
    let replace_preview = if effect.replace.len() > 100 {
        format!("{}...", &effect.replace[..100])
    } else {
        effect.replace.clone()
    };
    let replace_display = replace_preview.replace('\n', "\\n");
    println!("{}Replace: \"{}\"", indent, replace_display);

    let format_str = match effect.format {
        TextFormat::Plain => "plain",
        TextFormat::Markdown => "markdown",
        TextFormat::Html => "html",
    };
    if effect.format != TextFormat::Plain {
        println!("{}Format: {}", indent, format_str);
    }

    if let Some(mode) = &effect.force_mode {
        let mode_str = match mode {
            TextInjectMode::Keys => "keys",
            TextInjectMode::Clipboard => "clipboard",
        };
        println!("{}Force mode: {}", indent, mode_str);
    }

    // Print variables
    if !effect.vars.is_empty() {
        println!("{}Variables:", indent);
        for var in &effect.vars {
            print_variable_details(var, &format!("{}  ", indent));
        }
    }
}

fn print_variable_details(var: &Variable, indent: &str) {
    println!("{}- name: {}", indent, var.name);
    println!("{}  type: {}", indent, var.var_type);
    if !var.params.is_empty() {
        println!("{}  params: {:?}", indent, var.params);
    }
    if !var.depends_on.is_empty() {
        println!("{}  depends_on: {:?}", indent, var.depends_on);
    }
}

// JSON output structures
#[derive(Serialize)]
struct ExplainOutputJson {
    trigger: String,
    found: bool,
    selected: Option<MatchDetailsJson>,
    candidates: Vec<MatchDetailsJson>,
}

#[derive(Serialize)]
struct MatchDetailsJson {
    source_file: String,
    match_id: i32,
    label: Option<String>,
    enabled: bool,
    cause_type: String,
    triggers: Option<Vec<String>>,
    regex: Option<String>,
    effect_type: String,
    replace: Option<String>,
    image_path: Option<String>,
    variables: Vec<VariableJson>,
    is_selected: bool,
}

#[derive(Serialize)]
struct VariableJson {
    name: String,
    var_type: String,
}

fn print_json_output(trigger: &str, candidates: &[MatchCandidate], show_all: bool) -> Result<()> {
    let selected = candidates.iter().find(|c| c.is_selected);

    let selected_json = selected.map(|c| match_to_json(c));

    let candidates_json: Vec<MatchDetailsJson> = if show_all {
        candidates
            .iter()
            .filter(|c| !c.is_selected)
            .map(match_to_json)
            .collect()
    } else {
        vec![]
    };

    let output = ExplainOutputJson {
        trigger: trigger.to_string(),
        found: true,
        selected: selected_json,
        candidates: candidates_json,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn match_to_json(candidate: &MatchCandidate) -> MatchDetailsJson {
    let m = candidate.info.m;

    let (cause_type, triggers, regex) = match &m.cause {
        MatchCause::Trigger(cause) => ("trigger".to_string(), Some(cause.triggers.clone()), None),
        MatchCause::Regex(cause) => ("regex".to_string(), None, Some(cause.regex.clone())),
        MatchCause::None => ("none".to_string(), None, None),
    };

    let (effect_type, replace, image_path, variables) = match &m.effect {
        MatchEffect::Text(effect) => (
            "text".to_string(),
            Some(effect.replace.clone()),
            None,
            effect
                .vars
                .iter()
                .map(|v| VariableJson {
                    name: v.name.clone(),
                    var_type: v.var_type.clone(),
                })
                .collect(),
        ),
        MatchEffect::Image(effect) => {
            ("image".to_string(), None, Some(effect.path.clone()), vec![])
        }
        MatchEffect::None => ("none".to_string(), None, None, vec![]),
    };

    MatchDetailsJson {
        source_file: candidate.info.source_file.to_string(),
        match_id: m.id,
        label: m.label.clone(),
        enabled: m.enabled,
        cause_type,
        triggers,
        regex,
        effect_type,
        replace,
        image_path,
        variables,
        is_selected: candidate.is_selected,
    }
}

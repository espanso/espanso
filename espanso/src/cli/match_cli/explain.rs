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
use std::fmt::Write;

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

    let output = explain_output(
        ExplainOptions {
            trigger,
            show_all,
            json_output,
            app_properties: AppProperties { title, class, exec },
        },
        &*config_store,
        &*match_store,
    )?;

    print!("{output}");

    Ok(())
}

pub(crate) struct ExplainOptions<'a> {
    pub trigger: &'a str,
    pub show_all: bool,
    pub json_output: bool,
    pub app_properties: AppProperties<'a>,
}

pub(crate) fn explain_output(
    options: ExplainOptions<'_>,
    config_store: &dyn ConfigStore,
    match_store: &dyn MatchStore,
) -> Result<String> {
    let config = config_store.active(&options.app_properties);
    let matches_with_sources = match_store.query_with_sources(config.match_paths());

    let mut candidates: Vec<MatchCandidate> = Vec::new();
    for info in &matches_with_sources {
        if match_has_trigger(info.m, options.trigger) {
            candidates.push(MatchCandidate {
                info,
                is_selected: false,
            });
        }
    }

    if candidates.is_empty() {
        if options.json_output {
            return Ok(serde_json::to_string_pretty(&ExplainOutputJson {
                trigger: options.trigger.to_string(),
                found: false,
                selected: None,
                candidates: vec![],
            })?);
        }

        let mut output = String::new();
        writeln!(output, "Trigger: \"{}\"", options.trigger)?;
        writeln!(output)?;
        writeln!(output, "No matches found for this trigger.")?;
        return Ok(output);
    }

    candidates[0].is_selected = true;

    if options.json_output {
        render_json_output(options.trigger, &candidates, options.show_all)
    } else {
        let mut output = String::new();
        render_human_output(&mut output, options.trigger, &candidates, options.show_all)?;
        Ok(output)
    }
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

fn render_human_output(
    output: &mut String,
    trigger: &str,
    candidates: &[MatchCandidate],
    show_all: bool,
) -> std::fmt::Result {
    writeln!(output, "Trigger: \"{}\"", trigger)?;
    writeln!(output)?;

    // Print the selected match
    if let Some(selected) = candidates.iter().find(|c| c.is_selected) {
        writeln!(output, "Selected match:")?;
        render_match_details(output, selected.info, "  ")?;
    }

    // Print other candidates if --all flag is set
    if show_all && candidates.len() > 1 {
        writeln!(output)?;
        writeln!(output, "Other candidates (not selected):")?;
        for candidate in candidates.iter().filter(|c| !c.is_selected) {
            writeln!(output)?;
            writeln!(output, "  File: {}", candidate.info.source_file)?;
            render_match_details(output, candidate.info, "    ")?;
            writeln!(
                output,
                "    Reason not selected: lower priority (appears later in resolution order)"
            )?;
        }
    } else if candidates.len() > 1 {
        writeln!(output)?;
        writeln!(
            output,
            "Note: {} other candidate(s) found. Use --all to see them.",
            candidates.len() - 1
        )?;
    }

    Ok(())
}

fn render_match_details(output: &mut String, info: &MatchInfo, indent: &str) -> std::fmt::Result {
    let m = info.m;

    writeln!(output, "{}Defined in: {}", indent, info.source_file)?;
    writeln!(output, "{}Match ID: {}", indent, m.id)?;

    if let Some(label) = &m.label {
        writeln!(output, "{}Label: {}", indent, label)?;
    }

    // Print cause details
    match &m.cause {
        MatchCause::Trigger(cause) => {
            writeln!(output, "{}Type: trigger", indent)?;
            render_trigger_details(output, cause, indent)?;
        }
        MatchCause::Regex(cause) => {
            writeln!(output, "{}Type: regex", indent)?;
            writeln!(output, "{}Regex: {}", indent, cause.regex)?;
        }
        MatchCause::None => {
            writeln!(output, "{}Type: none", indent)?;
        }
    }

    // Print effect details
    match &m.effect {
        MatchEffect::Text(effect) => {
            writeln!(output, "{}Effect: text replacement", indent)?;
            render_text_effect_details(output, effect, indent)?;
        }
        MatchEffect::Image(effect) => {
            writeln!(output, "{}Effect: image", indent)?;
            writeln!(output, "{}Image path: {}", indent, effect.path)?;
        }
        MatchEffect::None => {
            writeln!(output, "{}Effect: none", indent)?;
        }
    }

    Ok(())
}

fn render_trigger_details(
    output: &mut String,
    cause: &TriggerCause,
    indent: &str,
) -> std::fmt::Result {
    if cause.triggers.len() == 1 {
        writeln!(output, "{}Trigger: {}", indent, cause.triggers[0])?;
    } else {
        writeln!(output, "{}Triggers: {:?}", indent, cause.triggers)?;
    }

    if cause.left_word || cause.right_word {
        let word_mode = match (cause.left_word, cause.right_word) {
            (true, true) => "both",
            (true, false) => "left",
            (false, true) => "right",
            _ => "none",
        };
        writeln!(output, "{}Word boundary: {}", indent, word_mode)?;
    }

    if cause.propagate_case {
        let style = match cause.uppercase_style {
            UpperCasingStyle::Uppercase => "uppercase",
            UpperCasingStyle::Capitalize => "capitalize",
            UpperCasingStyle::CapitalizeWords => "capitalize_words",
        };
        writeln!(output, "{}Propagate case: yes (style: {})", indent, style)?;
    }

    Ok(())
}

fn render_text_effect_details(
    output: &mut String,
    effect: &TextEffect,
    indent: &str,
) -> std::fmt::Result {
    // Truncate long replacements for display
    let replace_preview = if effect.replace.len() > 100 {
        format!("{}...", &effect.replace[..100])
    } else {
        effect.replace.clone()
    };
    let replace_display = replace_preview.replace('\n', "\\n");
    writeln!(output, "{}Replace: \"{}\"", indent, replace_display)?;

    let format_str = match effect.format {
        TextFormat::Plain => "plain",
        TextFormat::Markdown => "markdown",
        TextFormat::Html => "html",
    };
    if effect.format != TextFormat::Plain {
        writeln!(output, "{}Format: {}", indent, format_str)?;
    }

    if let Some(mode) = &effect.force_mode {
        let mode_str = match mode {
            TextInjectMode::Keys => "keys",
            TextInjectMode::Clipboard => "clipboard",
        };
        writeln!(output, "{}Force mode: {}", indent, mode_str)?;
    }

    // Print variables
    if !effect.vars.is_empty() {
        writeln!(output, "{}Variables:", indent)?;
        for var in &effect.vars {
            render_variable_details(output, var, &format!("{}  ", indent))?;
        }
    }

    Ok(())
}

fn render_variable_details(output: &mut String, var: &Variable, indent: &str) -> std::fmt::Result {
    writeln!(output, "{}- name: {}", indent, var.name)?;
    writeln!(output, "{}  type: {}", indent, var.var_type)?;
    if !var.params.is_empty() {
        writeln!(output, "{}  params: {:?}", indent, var.params)?;
    }
    if !var.depends_on.is_empty() {
        writeln!(output, "{}  depends_on: {:?}", indent, var.depends_on)?;
    }

    Ok(())
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

fn render_json_output(
    trigger: &str,
    candidates: &[MatchCandidate],
    show_all: bool,
) -> Result<String> {
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

    Ok(serde_json::to_string_pretty(&output)?)
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

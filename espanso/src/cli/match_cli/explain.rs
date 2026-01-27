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

use anyhow::{Context, Result};
use clap::ArgMatches;
use espanso_config::{
    config::{AppProperties, ConfigStore},
    matches::{
        store::{MatchInfo, MatchStore},
        Match, MatchCause, MatchEffect, TextEffect, TextFormat, TextInjectMode, TriggerCause,
        UpperCasingStyle, Variable,
    },
};
use espanso_render::{
    CasingStyle, Context as RenderContext, RenderOptions, RenderResult, Renderer, Template,
};
use serde::Serialize;
use std::fmt::Write;

use crate::path::resolve_paths;

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
    let match_set = match_store.query(config.match_paths());
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

    let paths = resolve_paths(None, None, None);
    let render_support = build_render_support(&match_set, &paths).ok();

    if options.json_output {
        render_json_output(
            options.trigger,
            &candidates,
            options.show_all,
            render_support.as_ref(),
        )
    } else {
        let mut output = String::new();
        render_human_output(
            &mut output,
            options.trigger,
            &candidates,
            options.show_all,
            render_support.as_ref(),
        )?;
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
    render_support: Option<&RenderSupport>,
) -> std::fmt::Result {
    writeln!(output, "Trigger: \"{}\"", trigger)?;
    writeln!(output)?;

    // Print the selected match
    if let Some(selected) = candidates.iter().find(|c| c.is_selected) {
        writeln!(output, "Selected match:")?;
        render_match_details(output, selected.info, trigger, "  ", render_support)?;
    }

    // Print other candidates if --all flag is set
    if show_all && candidates.len() > 1 {
        writeln!(output)?;
        writeln!(output, "Other candidates (not selected):")?;
        for candidate in candidates.iter().filter(|c| !c.is_selected) {
            writeln!(output)?;
            writeln!(output, "  File: {}", candidate.info.source_file)?;
            render_match_details(output, candidate.info, trigger, "    ", render_support)?;
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

fn render_match_details(
    output: &mut String,
    info: &MatchInfo,
    trigger: &str,
    indent: &str,
    render_support: Option<&RenderSupport>,
) -> std::fmt::Result {
    let m = info.m;
    let line_number = find_line_number(info);

    writeln!(output, "{}Defined in: {}", indent, info.source_file)?;
    if let Some(line_number) = line_number {
        writeln!(output, "{}Line number: {}", indent, line_number)?;
    } else {
        writeln!(output, "{}Line number: unknown", indent)?;
    }
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
            if let Some(render_support) = render_support {
                if let Some(current_output) = render_current_output(render_support, m, trigger) {
                    let display = escape_for_display(&current_output);
                    writeln!(output, "{}Current output: \"{}\"", indent, display)?;
                }
            }
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
    let replace_display = escape_for_display(&effect.replace);
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
    line_number: Option<usize>,
    match_id: i32,
    label: Option<String>,
    cause_type: String,
    triggers: Option<Vec<String>>,
    regex: Option<String>,
    effect_type: String,
    replace: Option<String>,
    image_path: Option<String>,
    variables: Vec<VariableJson>,
    current_output: Option<String>,
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
    render_support: Option<&RenderSupport>,
) -> Result<String> {
    let selected = candidates.iter().find(|c| c.is_selected);

    let selected_json = selected.map(|c| match_to_json(c, trigger, render_support));

    let candidates_json: Vec<MatchDetailsJson> = if show_all {
        candidates
            .iter()
            .filter(|c| !c.is_selected)
            .map(|c| match_to_json(c, trigger, render_support))
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

fn match_to_json(
    candidate: &MatchCandidate,
    trigger: &str,
    render_support: Option<&RenderSupport>,
) -> MatchDetailsJson {
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

    let current_output =
        render_support.and_then(|support| render_current_output(support, m, trigger));

    MatchDetailsJson {
        source_file: candidate.info.source_file.to_string(),
        line_number: find_line_number(candidate.info),
        match_id: m.id,
        label: m.label.clone(),
        cause_type,
        triggers,
        regex,
        effect_type,
        replace,
        image_path,
        variables,
        current_output,
        is_selected: candidate.is_selected,
    }
}

struct RenderSupport {
    templates: Vec<Template>,
    global_vars: Vec<espanso_render::Variable>,
    paths: crate::path::Paths,
}

fn build_render_support(
    match_set: &espanso_config::matches::store::MatchSet<'_>,
    paths: &crate::path::Paths,
) -> Result<RenderSupport> {
    let templates: Vec<Template> = match_set
        .matches
        .iter()
        .filter_map(|m| convert_to_template(m))
        .collect();

    let global_vars: Vec<espanso_render::Variable> = match_set
        .global_vars
        .iter()
        .copied()
        .map(convert_var)
        .collect();

    Ok(RenderSupport {
        templates,
        global_vars,
        paths: paths.clone(),
    })
}

fn render_current_output(support: &RenderSupport, m: &Match, trigger: &str) -> Option<String> {
    let template = convert_to_template(m)?;
    let template_refs: Vec<&Template> = support.templates.iter().collect();
    let global_var_refs: Vec<&espanso_render::Variable> = support.global_vars.iter().collect();
    let context = RenderContext {
        global_vars: global_var_refs,
        templates: template_refs,
    };
    let locale_provider = espanso_render::extension::date::DefaultLocaleProvider::new();
    let date_extension = espanso_render::extension::date::DateExtension::new(&locale_provider);
    let echo_extension = espanso_render::extension::echo::EchoExtension::new();
    let random_extension = espanso_render::extension::random::RandomExtension::new();
    let home_path = dirs::home_dir()
        .context("unable to obtain home dir path")
        .ok()?;
    let script_extension = espanso_render::extension::script::ScriptExtension::new(
        &support.paths.config,
        &home_path,
        &support.paths.packages,
    );
    let shell_extension =
        espanso_render::extension::shell::ShellExtension::new(&support.paths.config);
    let renderer = espanso_render::create(vec![
        &date_extension,
        &echo_extension,
        &random_extension,
        &script_extension,
        &shell_extension,
    ]);
    let options = RenderOptions {
        casing_style: calculate_casing_style(m, trigger),
    };
    match renderer.render(&template, &context, &options) {
        RenderResult::Success(body) => Some(body),
        RenderResult::Aborted => Some("Rendering aborted".to_string()),
        RenderResult::Error(err) => Some(format!("Rendering error: {err:?}")),
    }
}

fn convert_to_template(m: &Match) -> Option<Template> {
    if let MatchEffect::Text(text_effect) = &m.effect {
        let ids = if let MatchCause::Trigger(cause) = &m.cause {
            cause.triggers.clone()
        } else {
            Vec::new()
        };

        Some(Template {
            ids,
            body: text_effect.replace.clone(),
            vars: text_effect.vars.iter().map(convert_var).collect(),
        })
    } else {
        None
    }
}

fn convert_var(var: &espanso_config::matches::Variable) -> espanso_render::Variable {
    espanso_render::Variable {
        name: var.name.clone(),
        var_type: var.var_type.clone(),
        params: convert_params(var.params.clone()),
        inject_vars: var.inject_vars,
        depends_on: var.depends_on.clone(),
    }
}

fn convert_params(params: espanso_config::matches::Params) -> espanso_render::Params {
    let mut new_params = espanso_render::Params::new();
    for (key, value) in params {
        new_params.insert(key, convert_value(value));
    }
    new_params
}

fn convert_value(value: espanso_config::matches::Value) -> espanso_render::Value {
    match value {
        espanso_config::matches::Value::Null => espanso_render::Value::Null,
        espanso_config::matches::Value::Bool(v) => espanso_render::Value::Bool(v),
        espanso_config::matches::Value::Number(n) => match n {
            espanso_config::matches::Number::Integer(i) => {
                espanso_render::Value::Number(espanso_render::Number::Integer(i))
            }
            espanso_config::matches::Number::Float(f) => {
                espanso_render::Value::Number(espanso_render::Number::Float(f.into_inner()))
            }
        },
        espanso_config::matches::Value::String(s) => espanso_render::Value::String(s),
        espanso_config::matches::Value::Array(v) => {
            espanso_render::Value::Array(v.into_iter().map(convert_value).collect())
        }
        espanso_config::matches::Value::Object(params) => {
            espanso_render::Value::Object(convert_params(params))
        }
    }
}

fn calculate_casing_style(m: &Match, trigger: &str) -> CasingStyle {
    let MatchCause::Trigger(cause) = &m.cause else {
        return CasingStyle::None;
    };

    if !cause.propagate_case {
        return CasingStyle::None;
    }

    let mut first_alphabetic = None;
    let mut second_alphabetic = None;

    for c in trigger.chars() {
        if c.is_alphabetic() {
            if first_alphabetic.is_none() {
                first_alphabetic = Some(c);
            } else if second_alphabetic.is_none() {
                second_alphabetic = Some(c);
            } else {
                break;
            }
        }
    }

    match (first_alphabetic, second_alphabetic) {
        (Some(first), Some(second)) => {
            if first.is_uppercase() {
                if second.is_uppercase() {
                    CasingStyle::Uppercase
                } else {
                    CasingStyle::Capitalize
                }
            } else if second.is_uppercase() {
                CasingStyle::CapitalizeWords
            } else {
                CasingStyle::None
            }
        }
        _ => CasingStyle::None,
    }
}

fn escape_for_display(value: &str) -> String {
    value.replace('\n', "\\n").replace('\r', "\\r")
}

fn find_line_number(info: &MatchInfo) -> Option<usize> {
    let contents = std::fs::read_to_string(info.source_file).ok()?;
    let trigger_values: Vec<&str> = match &info.m.cause {
        MatchCause::Trigger(cause) => cause.triggers.iter().map(String::as_str).collect(),
        MatchCause::Regex(cause) => vec![cause.regex.as_str()],
        MatchCause::None => Vec::new(),
    };

    if trigger_values.is_empty() {
        return None;
    }

    for (index, line) in contents.lines().enumerate() {
        let trimmed = line.trim_start();
        let has_trigger = matches!(info.m.cause, MatchCause::Trigger(_))
            && trigger_values.iter().any(|trigger| line.contains(trigger))
            && (line.contains("trigger") || line.contains("triggers") || trimmed.starts_with('-'));
        let has_regex = matches!(info.m.cause, MatchCause::Regex(_))
            && trigger_values.iter().any(|pattern| line.contains(pattern))
            && line.contains("regex");

        if has_trigger || has_regex {
            return Some(index + 1);
        }
    }
    None
}

/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 * ... (license text continues)
 */

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use anyhow::Result;
use log::info;

use espanso_config::matches::store::{self, MatchSet, MatchStore};
use espanso_config::matches::{Match, MatchCause, MatchEffect};
use espanso_match::event::{Event, Key};
use espanso_match::regex::{RegexMatch, RegexMatcher, RegexMatcherOptions};
use espanso_match::rolling::matcher::{RollingMatcher, RollingMatcherOptions};
use espanso_match::rolling::{RollingMatch, StringMatchOptions};
use espanso_match::Matcher;

#[derive(Debug, Clone)]
pub struct TestResult {
    pub matched: bool,
    pub match_id: Option<i32>,
    pub match_trigger: Option<String>,
    pub match_file: Option<String>,
    pub match_line: Option<u32>,
    pub match_type: String,
    pub rendered_output: Option<String>,
    pub variable_trace: Vec<VariableTraceEntry>,
    pub elapsed_us: u64,
}

#[derive(Debug, Clone)]
pub struct VariableTraceEntry {
    pub var_name: String,
    pub var_type: String,
    pub input_params: HashMap<String, String>,
    pub output_value: String,
    pub error: Option<String>,
}

/// Run a test expansion against loaded config.
pub fn test_expansion(
    config_dir: &Path,
    input: &str,
    _app_title: Option<&str>,
    _app_class: Option<&str>,
    _app_exec: Option<&str>,
) -> Result<TestResult> {
    let start = Instant::now();

    info!("Testing expansion for input: '{}'", input);

    let (_config_store, match_store, _errors) = espanso_config::load(config_dir)?;
    let default_config = _config_store.default();
    let match_paths = default_config.match_paths();
    let match_set = match_store.query(match_paths);

    if input.is_empty() {
        return Ok(TestResult {
            matched: false,
            match_id: None,
            match_trigger: None,
            match_file: None,
            match_line: None,
            match_type: String::new(),
            rendered_output: None,
            variable_trace: Vec::new(),
            elapsed_us: start.elapsed().as_micros() as u64,
        });
    }

    // Build both matchers
    let (rolling, regex) = build_matchers(&match_set);

    // Run input through both
    let (rolling_hits, regex_hits) = run_matchers(input, &rolling, &regex);

    let best = rolling_hits
        .first()
        .map(|r| (r.id, r.trigger.clone(), r.vars.clone()))
        .or_else(|| {
            regex_hits
                .first()
                .map(|r| (r.id, r.trigger.clone(), r.vars.clone()))
        });

    let matched = best.is_some();
    let match_type = if !rolling_hits.is_empty() {
        "Rolling (Trie)".to_string()
    } else if !regex_hits.is_empty() {
        "Regex".to_string()
    } else {
        String::new()
    };

    let (match_id, match_trigger, rendered_output) = if let Some((id, trigger, vars)) = best {
        let body = match_set
            .matches
            .iter()
            .find(|m| m.id == id)
            .map(|m| match &m.effect {
                MatchEffect::Text(te) => te.replace.clone(),
                MatchEffect::Image(ie) => format!("[Image: {}]", ie.path),
                MatchEffect::None => String::new(),
            });
        (Some(id), Some(trigger), body)
    } else {
        (None, None, None)
    };

    Ok(TestResult {
        matched,
        match_id,
        match_trigger,
        match_file: None,
        match_line: None,
        match_type,
        rendered_output,
        variable_trace: Vec::new(),
        elapsed_us: start.elapsed().as_micros() as u64,
    })
}

fn build_matchers(
    match_set: &MatchSet,
) -> (RollingMatcher<i32>, RegexMatcher<i32>) {
    let mut rolling_items: Vec<RollingMatch<i32>> = Vec::new();
    let mut regex_items: Vec<RegexMatch<i32>> = Vec::new();

    for m in &match_set.matches {
        match &m.cause {
            MatchCause::Trigger(tc) => {
                for trigger in &tc.triggers {
                    let opt = StringMatchOptions {
                        case_insensitive: false,
                        left_word: tc.left_word,
                        right_word: tc.right_word,
                    };
                    rolling_items.push(RollingMatch::from_string(m.id, trigger, &opt));
                }
            }
            MatchCause::Regex(rc) => {
                regex_items.push(RegexMatch::new(m.id, &rc.regex));
            }
            MatchCause::None => {}
        }
    }

    let rolling = RollingMatcher::new(&rolling_items, RollingMatcherOptions::default());
    let regex = RegexMatcher::new(&regex_items, RegexMatcherOptions::default());

    (rolling, regex)
}

fn run_matchers(
    input: &str,
    rolling: &RollingMatcher<i32>,
    regex: &RegexMatcher<i32>,
) -> (
    Vec<espanso_match::MatchResult<i32>>,
    Vec<espanso_match::MatchResult<i32>>,
) {
    let mut rolling_state = None;
    let mut regex_state = None;
    let mut rolling_results = Vec::new();
    let mut regex_results = Vec::new();

    for ch in input.chars() {
        let event = Event::Key {
            key: Key::Other,
            chars: Some(ch.to_string()),
        };

        // Rolling matcher
        {
            let (new_state, results) = rolling.process(rolling_state.as_ref(), event.clone());
            rolling_state = Some(new_state);
            if !results.is_empty() {
                rolling_results = results;
                break;
            }
        }

        // Regex matcher
        {
            let (new_state, results) = regex.process(regex_state.as_ref(), event);
            regex_state = Some(new_state);
            if !results.is_empty() {
                regex_results = results;
                break;
            }
        }
    }

    (rolling_results, regex_results)
}

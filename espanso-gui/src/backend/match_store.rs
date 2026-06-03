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

//! In-memory cache of matches for the GUI.

use std::path::PathBuf;

use log::{info, warn};

use crate::backend::config_io::{self, MatchEntry};

#[derive(Debug, Clone)]
pub struct GuiMatch {
    pub id: i32,
    pub triggers: Vec<String>,
    pub trigger_display: String,
    pub replace_preview: String,
    pub match_type: MatchType,
    pub source_file: String,
    pub source_line: u32,
    pub label: Option<String>,
    pub search_terms: Vec<String>,
    pub is_image: bool,
    pub image_path: Option<String>,
    pub is_word: bool,
    pub propagate_case: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchType {
    Text,
    Image,
    Markdown,
    Html,
    Regex,
    Script,
}

impl MatchType {
    pub fn label(&self) -> &'static str {
        match self {
            MatchType::Text => "Text",
            MatchType::Image => "Image",
            MatchType::Markdown => "Markdown",
            MatchType::Html => "HTML",
            MatchType::Regex => "Regex",
            MatchType::Script => "Script",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchFilter {
    pub search_query: String,
    pub type_filter: Option<MatchType>,
}

impl Default for MatchFilter {
    fn default() -> Self {
        MatchFilter {
            search_query: String::new(),
            type_filter: None,
        }
    }
}

pub struct MatchCache {
    pub all_matches: Vec<GuiMatch>,
    config_dir: Option<PathBuf>,
    needs_reload: bool,
}

impl MatchCache {
    pub fn new(config_dir: Option<PathBuf>) -> Self {
        MatchCache {
            all_matches: Vec::new(),
            config_dir,
            needs_reload: true,
        }
    }

    pub fn ensure_loaded(&mut self) {
        if self.needs_reload {
            self.reload();
        }
    }

    pub fn reload(&mut self) {
        self.all_matches.clear();

        if let Some(ref config_dir) = self.config_dir {
            match config_io::load_all(config_dir) {
                Ok((config_store, match_store)) => {
                    let entries = config_io::list_matches(
                        config_store.as_ref(),
                        match_store.as_ref(),
                    );
                    for entry in entries {
                        self.all_matches.push(convert_match_entry(&entry));
                    }
                    info!("Loaded {} matches", self.all_matches.len());
                }
                Err(e) => {
                    warn!("Failed to load config: {}", e);
                }
            }
        }

        self.needs_reload = false;
    }

    pub fn mark_stale(&mut self) {
        self.needs_reload = true;
    }

    pub fn filtered(&mut self, filter: &MatchFilter) -> Vec<GuiMatch> {
        self.ensure_loaded();

        let query_lower = filter.search_query.to_lowercase();

        self.all_matches
            .iter()
            .filter(|m| {
                if let Some(type_filter) = &filter.type_filter {
                    if m.match_type != *type_filter {
                        return false;
                    }
                }
                if !query_lower.is_empty() {
                    let in_triggers = m
                        .triggers
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower));
                    let in_replace = m.replace_preview.to_lowercase().contains(&query_lower);
                    let in_label = m
                        .label
                        .as_ref()
                        .map(|l| l.to_lowercase().contains(&query_lower))
                        .unwrap_or(false);
                    let in_search_terms = m
                        .search_terms
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower));
                    in_triggers || in_replace || in_label || in_search_terms
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }

    pub fn get(&mut self, id: i32) -> Option<&GuiMatch> {
        self.ensure_loaded();
        self.all_matches.iter().find(|m| m.id == id)
    }
}

fn convert_match_entry(entry: &MatchEntry) -> GuiMatch {
    use espanso_config::matches::MatchCause;
    use espanso_config::matches::MatchEffect;

    let triggers = match &entry.m.cause {
        MatchCause::Trigger(tc) => tc.triggers.clone(),
        MatchCause::Regex(rc) => vec![format!("/{}/", rc.regex)],
        MatchCause::None => vec![],
    };

    let trigger_display = triggers
        .first()
        .cloned()
        .unwrap_or_else(|| "(no trigger)".to_string());

    let (replace_preview, match_type, is_image, image_path) = match &entry.m.effect {
        MatchEffect::Text(te) => {
            let preview = if te.replace.len() > 60 {
                format!("{}...", &te.replace[..57])
            } else {
                te.replace.clone()
            };
            let mtype = if te.vars.is_empty() {
                MatchType::Text
            } else {
                MatchType::Script
            };
            (preview, mtype, false, None)
        }
        MatchEffect::Image(ie) => {
            (ie.path.clone(), MatchType::Image, true, Some(ie.path.clone()))
        }
        MatchEffect::None => (String::new(), MatchType::Text, false, None),
    };

    let is_word = match &entry.m.cause {
        MatchCause::Trigger(tc) => tc.left_word || tc.right_word,
        _ => false,
    };

    let propagate_case = match &entry.m.cause {
        MatchCause::Trigger(tc) => tc.propagate_case,
        _ => false,
    };

    GuiMatch {
        id: entry.m.id,
        triggers,
        trigger_display,
        replace_preview,
        match_type,
        source_file: entry.source_file.clone(),
        source_line: entry.source_line,
        label: entry.m.label.clone(),
        search_terms: entry.m.search_terms.clone(),
        is_image,
        image_path,
        is_word,
        propagate_case,
    }
}

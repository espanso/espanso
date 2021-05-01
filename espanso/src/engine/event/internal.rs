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

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct MatchesDetectedEvent {
  pub matches: Vec<DetectedMatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DetectedMatch {
  pub id: i32,
  pub trigger: Option<String>,
  pub left_separator: Option<String>,
  pub right_separator: Option<String>,
  pub args: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchSelectedEvent {
  pub chosen: DetectedMatch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CauseCompensatedMatchEvent {
  pub m: DetectedMatch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderingRequestedEvent {
  pub match_id: i32,
  pub trigger: Option<String>,
  pub left_separator: Option<String>,
  pub right_separator: Option<String>,
  pub trigger_args: HashMap<String, String>,
  pub format: TextFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextFormat {
  Plain,
  Markdown,
  Html, 
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageRequestedEvent {
  pub match_id: i32,
  pub image_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageResolvedEvent {
  pub image_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderedEvent {
  pub match_id: i32,
  pub body: String,
  pub format: TextFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscardPreviousEvent {
  // All Events with a source_id smaller than this one will be discarded
  pub minimum_source_id: u32,
}
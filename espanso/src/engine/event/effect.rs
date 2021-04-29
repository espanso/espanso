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

use super::input::Key;

#[derive(Debug, Clone, PartialEq)]
pub struct TriggerCompensationEvent {
  pub trigger: String,
  pub left_separator: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CursorHintCompensationEvent {
  pub cursor_hint_back_count: usize,
}

#[derive(Debug, Clone)]
pub struct TextInjectRequest {
  pub text: String,
  pub force_mode: Option<TextInjectMode>,
}

#[derive(Debug, Clone)]
pub struct MarkdownInjectRequest {
  pub markdown: String,
}

#[derive(Debug, Clone)]
pub struct HtmlInjectRequest {
  pub html: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TextInjectMode {
  Keys,
  Clipboard,
}

#[derive(Debug, Clone)]
pub struct KeySequenceInjectRequest {
  pub keys: Vec<Key>,
}
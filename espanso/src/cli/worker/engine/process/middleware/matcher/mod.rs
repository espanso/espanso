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

use espanso_engine::{
    event::input::Key,
    process::{MatchResult, MatcherEvent},
};
use espanso_match::regex::RegexMatcherState;
use espanso_match::rolling::matcher::RollingMatcherState;

use enum_as_inner::EnumAsInner;

pub mod convert;
pub mod regex;
pub mod rolling;

#[derive(Clone, EnumAsInner)]
pub enum MatcherState<'a> {
    Rolling(RollingMatcherState<'a, i32>),
    Regex(RegexMatcherState),
}

pub fn convert_to_match_event(event: &MatcherEvent) -> espanso_match::event::Event {
    match event {
        MatcherEvent::Key { key, chars } => espanso_match::event::Event::Key {
            key: convert_to_match_key(key.clone()),
            chars: chars.to_owned(),
        },
        MatcherEvent::VirtualSeparator => espanso_match::event::Event::VirtualSeparator,
    }
}

pub fn convert_to_engine_result(result: espanso_match::MatchResult<i32>) -> MatchResult {
    MatchResult {
        id: result.id,
        trigger: result.trigger,
        left_separator: result.left_separator,
        right_separator: result.right_separator,
        args: result.vars,
    }
}

pub fn convert_to_match_key(key: Key) -> espanso_match::event::Key {
    match key {
        Key::Alt => espanso_match::event::Key::Alt,
        Key::CapsLock => espanso_match::event::Key::CapsLock,
        Key::Control => espanso_match::event::Key::Control,
        Key::Meta => espanso_match::event::Key::Meta,
        Key::NumLock => espanso_match::event::Key::NumLock,
        Key::Shift => espanso_match::event::Key::Shift,
        Key::Enter => espanso_match::event::Key::Enter,
        Key::Tab => espanso_match::event::Key::Tab,
        Key::Space => espanso_match::event::Key::Space,
        Key::ArrowDown => espanso_match::event::Key::ArrowDown,
        Key::ArrowLeft => espanso_match::event::Key::ArrowLeft,
        Key::ArrowRight => espanso_match::event::Key::ArrowRight,
        Key::ArrowUp => espanso_match::event::Key::ArrowUp,
        Key::End => espanso_match::event::Key::End,
        Key::Home => espanso_match::event::Key::Home,
        Key::PageDown => espanso_match::event::Key::PageDown,
        Key::PageUp => espanso_match::event::Key::PageUp,
        Key::Escape => espanso_match::event::Key::Escape,
        Key::Backspace => espanso_match::event::Key::Backspace,
        Key::F1 => espanso_match::event::Key::F1,
        Key::F2 => espanso_match::event::Key::F2,
        Key::F3 => espanso_match::event::Key::F3,
        Key::F4 => espanso_match::event::Key::F4,
        Key::F5 => espanso_match::event::Key::F5,
        Key::F6 => espanso_match::event::Key::F6,
        Key::F7 => espanso_match::event::Key::F7,
        Key::F8 => espanso_match::event::Key::F8,
        Key::F9 => espanso_match::event::Key::F9,
        Key::F10 => espanso_match::event::Key::F10,
        Key::F11 => espanso_match::event::Key::F11,
        Key::F12 => espanso_match::event::Key::F12,
        Key::F13 => espanso_match::event::Key::F13,
        Key::F14 => espanso_match::event::Key::F14,
        Key::F15 => espanso_match::event::Key::F15,
        Key::F16 => espanso_match::event::Key::F16,
        Key::F17 => espanso_match::event::Key::F17,
        Key::F18 => espanso_match::event::Key::F18,
        Key::F19 => espanso_match::event::Key::F19,
        Key::F20 => espanso_match::event::Key::F20,
        Key::Other(_) => espanso_match::event::Key::Other,
        _ => espanso_match::event::Key::Other,
    }
}

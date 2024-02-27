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

use crate::{
    dispatch::Executor,
    event::{effect::TextInjectMode, Event, EventType},
};
use anyhow::Result;
use log::{error, trace};

pub trait TextInjector {
    fn name(&self) -> &'static str;
    fn inject_text(&self, text: &str) -> Result<()>;
}

pub trait ModeProvider {
    fn active_mode(&self) -> Mode;
}

pub enum Mode {
    Event,
    Clipboard,
    Auto {
        // Maximum size after which the clipboard backend
        // is used over the event one to speed up the injection.
        clipboard_threshold: usize,
    },
}

pub struct TextInjectExecutor<'a> {
    event_injector: &'a dyn TextInjector,
    clipboard_injector: &'a dyn TextInjector,
    mode_provider: &'a dyn ModeProvider,
}

impl<'a> TextInjectExecutor<'a> {
    pub fn new(
        event_injector: &'a dyn TextInjector,
        clipboard_injector: &'a dyn TextInjector,
        mode_provider: &'a dyn ModeProvider,
    ) -> Self {
        Self {
            event_injector,
            clipboard_injector,
            mode_provider,
        }
    }
}

impl<'a> Executor for TextInjectExecutor<'a> {
    fn execute(&self, event: &Event) -> bool {
        if let EventType::TextInject(inject_event) = &event.etype {
            let active_mode = self.mode_provider.active_mode();

            let injector = if let Some(force_mode) = &inject_event.force_mode {
                if let TextInjectMode::Keys = force_mode {
                    self.event_injector
                } else {
                    self.clipboard_injector
                }
            } else if let Mode::Clipboard = active_mode {
                self.clipboard_injector
            } else if let Mode::Event = active_mode {
                self.event_injector
            } else if let Mode::Auto {
                clipboard_threshold,
            } = active_mode
            {
                if inject_event.text.chars().count() > clipboard_threshold {
                    self.clipboard_injector
                } else if cfg!(target_os = "linux") {
                    if inject_event.text.chars().all(|c| c.is_ascii()) {
                        self.event_injector
                    } else {
                        self.clipboard_injector
                    }
                } else {
                    self.event_injector
                }
            } else {
                self.event_injector
            };

            trace!("using injector: {}", injector.name());

            if let Err(error) = injector.inject_text(&inject_event.text) {
                error!(
                    "text injector ({}) reported an error: {:?}",
                    injector.name(),
                    error
                );
            }

            return true;
        }

        false
    }
}

// TODO: test

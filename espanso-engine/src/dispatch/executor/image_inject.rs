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
use log::error;

use crate::{
    dispatch::Executor,
    event::{Event, EventType},
};

pub trait ImageInjector {
    fn inject_image(&self, path: &str) -> Result<()>;
}

pub struct ImageInjectExecutor<'a> {
    injector: &'a dyn ImageInjector,
}

impl<'a> ImageInjectExecutor<'a> {
    pub fn new(injector: &'a dyn ImageInjector) -> Self {
        Self { injector }
    }
}

impl<'a> Executor for ImageInjectExecutor<'a> {
    fn execute(&self, event: &Event) -> bool {
        if let EventType::ImageInject(inject_event) = &event.etype {
            if let Err(error) = self.injector.inject_image(&inject_event.image_path) {
                error!("image injector reported an error: {:?}", error);
            }

            return true;
        }

        false
    }
}

// TODO: test

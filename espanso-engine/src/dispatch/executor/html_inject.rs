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

pub trait HtmlInjector {
  fn inject_html(&self, html: &str, fallback: &str) -> Result<()>;
}

pub struct HtmlInjectExecutor<'a> {
  injector: &'a dyn HtmlInjector,
}

impl<'a> HtmlInjectExecutor<'a> {
  pub fn new(injector: &'a dyn HtmlInjector) -> Self {
    Self { injector }
  }
}

impl<'a> Executor for HtmlInjectExecutor<'a> {
  fn execute(&self, event: &Event) -> bool {
    if let EventType::HtmlInject(inject_event) = &event.etype {
      // Render the text fallback for those applications that don't support HTML clipboard
      let decorator = html2text::render::text_renderer::TrivialDecorator::new();
      let fallback_text =
        html2text::from_read_with_decorator(inject_event.html.as_bytes(), 1000000, decorator);

      if let Err(error) = self
        .injector
        .inject_html(&inject_event.html, &fallback_text)
      {
        error!("html injector reported an error: {:?}", error);
      }

      return true;
    }

    false
  }
}

// TODO: test

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

use super::{
  ContextMenuHandler, Event, IconHandler, ImageInjector, SecureInputManager, TextUIHandler,
};
use super::{Dispatcher, Executor, HtmlInjector, KeyInjector, ModeProvider, TextInjector};

pub struct DefaultDispatcher<'a> {
  executors: Vec<Box<dyn Executor + 'a>>,
}

#[allow(clippy::too_many_arguments)]
impl<'a> DefaultDispatcher<'a> {
  pub fn new(
    event_injector: &'a dyn TextInjector,
    clipboard_injector: &'a dyn TextInjector,
    mode_provider: &'a dyn ModeProvider,
    key_injector: &'a dyn KeyInjector,
    html_injector: &'a dyn HtmlInjector,
    image_injector: &'a dyn ImageInjector,
    context_menu_handler: &'a dyn ContextMenuHandler,
    icon_handler: &'a dyn IconHandler,
    secure_input_manager: &'a dyn SecureInputManager,
    text_ui_handler: &'a dyn TextUIHandler,
  ) -> Self {
    Self {
      executors: vec![
        Box::new(super::executor::text_inject::TextInjectExecutor::new(
          event_injector,
          clipboard_injector,
          mode_provider,
        )),
        Box::new(super::executor::key_inject::KeyInjectExecutor::new(
          key_injector,
        )),
        Box::new(super::executor::html_inject::HtmlInjectExecutor::new(
          html_injector,
        )),
        Box::new(super::executor::image_inject::ImageInjectExecutor::new(
          image_injector,
        )),
        Box::new(super::executor::context_menu::ContextMenuExecutor::new(
          context_menu_handler,
        )),
        Box::new(super::executor::icon_update::IconUpdateExecutor::new(
          icon_handler,
        )),
        Box::new(super::executor::secure_input::SecureInputExecutor::new(
          secure_input_manager,
        )),
        Box::new(super::executor::text_ui::TextUIExecutor::new(
          text_ui_handler,
        )),
      ],
    }
  }
}

impl<'a> Dispatcher for DefaultDispatcher<'a> {
  fn dispatch(&self, event: Event) {
    for executor in self.executors.iter() {
      if executor.execute(&event) {
        break;
      }
    }
  }
}

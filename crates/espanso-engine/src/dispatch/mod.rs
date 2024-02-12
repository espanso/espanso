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

use crate::event::Event;

mod default;
mod executor;

pub trait Executor {
  fn execute(&self, event: &Event) -> bool;
}

pub trait Dispatcher {
  fn dispatch(&self, event: Event);
}

// Re-export dependency injection entities
pub use executor::context_menu::ContextMenuHandler;
pub use executor::html_inject::HtmlInjector;
pub use executor::icon_update::IconHandler;
pub use executor::image_inject::ImageInjector;
pub use executor::key_inject::KeyInjector;
pub use executor::secure_input::SecureInputManager;
pub use executor::text_inject::{Mode, ModeProvider, TextInjector};
pub use executor::text_ui::{TextUIExecutor, TextUIHandler};

#[allow(clippy::too_many_arguments)]
pub fn default<'a>(
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
) -> impl Dispatcher + 'a {
  default::DefaultDispatcher::new(
    event_injector,
    clipboard_injector,
    mode_provider,
    key_injector,
    html_injector,
    image_injector,
    context_menu_handler,
    icon_handler,
    secure_input_manager,
    text_ui_handler,
  )
}

/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

pub use crate::sys::welcome::show;

pub struct WelcomeOptions {
  pub window_icon_path: Option<String>,
  pub tray_image_path: Option<String>,

  pub handlers: WelcomeHandlers,
}

pub struct WelcomeHandlers {
  pub dont_show_again_changed: Option<Box<dyn Fn(bool) + Send>>,
}

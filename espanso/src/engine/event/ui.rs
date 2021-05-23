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

#[derive(Debug, Clone, PartialEq)]
pub struct ShowContextMenuEvent {
  pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem {
  Simple(SimpleMenuItem),
  Sub(SubMenuItem),
  Separator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleMenuItem {
  pub id: u32,
  pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubMenuItem {
  pub label: String,
  pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IconStatusChangeEvent {
  pub status: IconStatus
}

#[derive(Debug, Clone, PartialEq)]
pub enum IconStatus {
  Enabled,
  Disabled,
  SecureInputDisabled,
}
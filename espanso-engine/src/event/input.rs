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

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
  Pressed,
  Released,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Variant {
  Left,
  Right,
}

#[derive(Debug, PartialEq, Clone)]
pub struct KeyboardEvent {
  pub key: Key,
  pub value: Option<String>,
  pub status: Status,
  pub variant: Option<Variant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
  Left,
  Right,
  Middle,
  Button1,
  Button2,
  Button3,
  Button4,
  Button5,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MouseEvent {
  pub button: MouseButton,
  pub status: Status,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
  // Modifiers
  Alt,
  CapsLock,
  Control,
  Meta,
  NumLock,
  Shift,

  // Whitespace
  Enter,
  Tab,
  Space,

  // Navigation
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowUp,
  End,
  Home,
  PageDown,
  PageUp,

  // UI
  Escape,

  // Editing keys
  Backspace,

  // Function keys
  F1,
  F2,
  F3,
  F4,
  F5,
  F6,
  F7,
  F8,
  F9,
  F10,
  F11,
  F12,
  F13,
  F14,
  F15,
  F16,
  F17,
  F18,
  F19,
  F20,

  // Other keys, includes the raw code provided by the operating system
  Other(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenuClickedEvent {
  pub context_item_id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HotKeyEvent {
  pub hotkey_id: i32,
}

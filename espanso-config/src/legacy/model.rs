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

use serde::{Deserialize, Serialize};

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub enum KeyModifier {
  CTRL,
  SHIFT,
  ALT,
  META,
  BACKSPACE,
  OFF,

  // These are specific variants of the ones above. See issue: #117
  // https://github.com/espanso/espanso/issues/117
  LEFT_CTRL,
  RIGHT_CTRL,
  LEFT_ALT,
  RIGHT_ALT,
  LEFT_META,
  RIGHT_META,
  LEFT_SHIFT,
  RIGHT_SHIFT,

  // Special cases, should not be used in config
  CAPS_LOCK,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum PasteShortcut {
  #[default]
  Default, // Default one for the current system
  CtrlV,       // Classic Ctrl+V shortcut
  CtrlShiftV,  // Could be used to paste without formatting in many applications
  ShiftInsert, // Often used in Linux systems
  CtrlAltV,    // Used in some Linux terminals (urxvt)
  MetaV,       // Corresponding to Win+V on Windows and Linux, CMD+V on macOS
}

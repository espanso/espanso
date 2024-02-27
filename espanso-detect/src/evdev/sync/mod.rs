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

#[derive(Debug, Clone, Copy)]
pub struct ModifiersState {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub caps_lock: bool,
    pub meta: bool,
    pub num_lock: bool,
}

#[cfg(feature = "wayland")]
mod wayland;
#[cfg(feature = "wayland")]
pub use wayland::get_modifiers_state;

#[cfg(not(feature = "wayland"))]
pub fn get_modifiers_state() -> anyhow::Result<Option<ModifiersState>> {
    // Fallback for non-wayland systems
    Ok(None)
}

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

use espanso_engine::process::InputMethodProvider;

pub struct InputMethodProviderAdapter;

impl InputMethodProvider for InputMethodProviderAdapter {
    #[cfg(target_os = "macos")]
    fn is_hangul_dubeolsik_active(&self) -> bool {
        espanso_mac_utils::is_hangul_dubeolsik_active()
    }

    #[cfg(not(target_os = "macos"))]
    fn is_hangul_dubeolsik_active(&self) -> bool {
        false
    }
}

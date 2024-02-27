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

#[cfg(feature = "modulo")]
pub fn show_welcome_screen() {
    let espanso_exe_path = std::env::current_exe().expect("unable to determine executable path");
    let mut command = std::process::Command::new(espanso_exe_path.to_string_lossy().to_string());
    command.args(["modulo", "welcome"]);

    command.spawn().expect("unable to show welcome screen");
}

#[cfg(not(feature = "modulo"))]
pub fn show_welcome_screen() {
    // NOOP
}

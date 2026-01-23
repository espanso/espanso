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

pub fn show(
    icon_path: Option<&str>,
    validate_import_data: extern "C" fn(
        data: *const std::os::raw::c_char,
        config_status: *mut std::os::raw::c_int,
        matches_status: *mut std::os::raw::c_int,
        packages_status: *mut std::os::raw::c_int,
    ) -> std::os::raw::c_int,
    perform_import: extern "C" fn(
        data: *const std::os::raw::c_char,
        import_config: std::os::raw::c_int,
        import_matches: std::os::raw::c_int,
        import_packages: std::os::raw::c_int,
        clear_config: std::os::raw::c_int,
        clear_matches: std::os::raw::c_int,
        clear_packages: std::os::raw::c_int,
    ) -> std::os::raw::c_int,
) {
    crate::sys::import_dialog::show(icon_path, validate_import_data, perform_import);
}

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

use std::os::raw::c_char;

#[link(name = "espansomacutils", kind = "static")]
extern "C" {
  pub fn mac_utils_get_secure_input_process(pid: *mut i64) -> i32;
  pub fn mac_utils_get_path_from_pid(pid: i64, buffer: *mut c_char, size: i32) -> i32;
  pub fn mac_utils_check_accessibility() -> i32;
  pub fn mac_utils_prompt_accessibility() -> i32;
  pub fn mac_utils_transition_to_foreground_app();
  pub fn mac_utils_transition_to_background_app();
  pub fn mac_utils_start_headless_eventloop();
  pub fn mac_utils_exit_headless_eventloop();
}

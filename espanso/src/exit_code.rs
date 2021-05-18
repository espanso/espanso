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

pub const WORKER_SUCCESS: i32 = 0;
pub const WORKER_ALREADY_RUNNING: i32 = 1;
pub const WORKER_GENERAL_ERROR: i32 = 2;
pub const WORKER_EXIT_ALL_PROCESSES: i32 = 101;
pub const WORKER_RESTART: i32 = 102;

pub const DAEMON_SUCCESS: i32 = 0;
pub const DAEMON_ALREADY_RUNNING: i32 = 1;
pub const DAEMON_GENERAL_ERROR: i32 = 2;
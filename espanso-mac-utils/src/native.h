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

#ifndef ESPANSO_MAC_UTILS_H
#define ESPANSO_MAC_UTILS_H

#include <stdint.h>

// If a process is currently holding `SecureInput`, then return 1 and set the pid pointer to the corresponding PID.
extern "C" int32_t mac_utils_get_secure_input_process(int64_t *pid);

// Find the executable path corresponding to the given PID, return 0 if no process was found.
extern "C" int32_t mac_utils_get_path_from_pid(int64_t pid, char *buff, int buff_size);

// Return 1 if the accessibility permissions have been granted, 0 otherwise
extern "C" int32_t mac_utils_check_accessibility();

// Return 1 if the accessibility permissions have been granted, 0 otherwise
extern "C" int32_t mac_utils_prompt_accessibility();

// When called, convert the current process to a foreground app (showing the dock icon).
extern "C" void mac_utils_transition_to_foreground_app();

// When called, convert the current process to a background app (hide the dock icon).
extern "C" void mac_utils_transition_to_background_app();

// Start and stop a "headless" eventloop to receive NSApplication events.
extern "C" void mac_utils_start_headless_eventloop();
extern "C" void mac_utils_exit_headless_eventloop();

#endif //ESPANSO_MAC_UTILS_H
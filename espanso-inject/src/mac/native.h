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

#ifndef ESPANSO_INJECT_H
#define ESPANSO_INJECT_H

#include <stdint.h>

// Inject a complete string using the KEYEVENTF_UNICODE flag
extern "C" void inject_string(char * string, int32_t delay);

// Send a sequence of vkey presses and releases
extern "C" void inject_separate_vkeys(int32_t *vkey_array, int32_t vkey_count, int32_t delay);

// Send a combination of vkeys, first pressing all the vkeys and then releasing
// This is needed for keyboard shortcuts, for example.
extern "C" void inject_vkeys_combination(int32_t *vkey_array, int32_t vkey_count, int32_t delay);

#endif //ESPANSO_INJECT_H
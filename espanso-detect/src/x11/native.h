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

#ifndef ESPANSO_DETECT_H
#define ESPANSO_DETECT_H

#include <stdint.h>

#define INPUT_EVENT_TYPE_KEYBOARD 1
#define INPUT_EVENT_TYPE_MOUSE 2

#define INPUT_STATUS_PRESSED 1
#define INPUT_STATUS_RELEASED 2

typedef struct
{
  // Keyboard or Mouse event
  int32_t event_type;

  // Contains the string corresponding to the key, if any
  char buffer[24];
  // Length of the extracted string. Equals 0 if no string is extracted
  int32_t buffer_len;

  // Code of the pressed key.
  int32_t key_sym;

  // Virtual key code of the pressed key in case of keyboard events
  // Mouse button code otherwise.
  int32_t key_code;

  // Pressed or Released status
  int32_t status;
} InputEvent;

typedef void (*EventCallback)(void *rust_istance, InputEvent data);

// Check if a X11 context is available, returning a non-zero code if true.
extern "C" int32_t detect_check_x11();

// Initialize the XRecord API and return the context pointer
extern "C" void *detect_initialize(void *rust_istance, int32_t *error_code);

// Run the event loop. Blocking call.
extern "C" int32_t detect_eventloop(void *context, EventCallback callback);

// Unregister from the XRecord API and destroy the context.
extern "C" int32_t detect_destroy(void *context);

#endif //ESPANSO_DETECT_H
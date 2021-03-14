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
#define INPUT_EVENT_TYPE_MOUSE    2
#define INPUT_EVENT_TYPE_HOTKEY   3

#define INPUT_STATUS_PRESSED      1
#define INPUT_STATUS_RELEASED     2

#define INPUT_LEFT_VARIANT 1
#define INPUT_RIGHT_VARIANT 2

#define INPUT_MOUSE_LEFT_BUTTON   1
#define INPUT_MOUSE_RIGHT_BUTTON  2
#define INPUT_MOUSE_MIDDLE_BUTTON 3
#define INPUT_MOUSE_BUTTON_1      4
#define INPUT_MOUSE_BUTTON_2      5
#define INPUT_MOUSE_BUTTON_3      6
#define INPUT_MOUSE_BUTTON_4      7
#define INPUT_MOUSE_BUTTON_5      8

typedef struct {
  // Keyboard or Mouse event
  int32_t event_type;

  // Contains the string corresponding to the key, if any
  uint16_t buffer[24];
  // Length of the extracted string. Equals 0 if no string is extracted
  int32_t buffer_len;
  
  // Virtual key code of the pressed key in case of keyboard events
  // Mouse button code for mouse events.
  // Hotkey id for hotkey events
  int32_t key_code;
  
  // Left or Right variant
  int32_t variant;

  // Pressed or Released status
  int32_t status;
} InputEvent;

typedef struct {
  int32_t hk_id;
  uint32_t key_code;
  uint32_t flags;
} HotKey;

typedef void (*EventCallback)(void * rust_istance, InputEvent data);


// Initialize the Raw Input API and the Window.
extern "C" void * detect_initialize(void * rust_istance, int32_t *error_code);

// Register the given hotkey, return a non-zero code if successful
extern "C" int32_t detect_register_hotkey(void * window, HotKey hotkey);

// Run the event loop. Blocking call.
extern "C" int32_t detect_eventloop(void * window, EventCallback callback);

// Destroy the given window.
extern "C" int32_t detect_destroy(void * window);

#endif //ESPANSO_DETECT_H
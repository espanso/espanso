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

typedef struct {
  // Keyboard, Mouse or Hotkey event
  int32_t event_type;

  // Contains the string corresponding to the key, if any
  char buffer[24];
  // Length of the extracted string. Equals 0 if no string is extracted
  int32_t buffer_len;
  
  // Virtual key code of the pressed key in case of keyboard events
  // Mouse button code if mouse_event.
  // Hotkey ID in case of hotkeys
  int32_t key_code;
  
  // Pressed or Released status
  int32_t status;

  // Modifier keys status, this is needed to "correct" missing modifier release events.
  // For more info, see the following issues:
  // https://github.com/espanso/espanso/issues/825
  // https://github.com/espanso/espanso/issues/858
  int32_t is_caps_lock_pressed;
  int32_t is_shift_pressed;
  int32_t is_control_pressed;
  int32_t is_option_pressed;
  int32_t is_command_pressed;
} InputEvent;

typedef void (*EventCallback)(InputEvent data);

typedef struct {
  int32_t hk_id;
  uint16_t key_code;
  uint32_t flags;
} HotKey;

typedef struct {
  HotKey *hotkeys;
  int32_t hotkeys_count;
} InitializeOptions;

// Initialize the event global monitor
extern "C" void * detect_initialize(EventCallback callback, InitializeOptions options);

#endif //ESPANSO_DETECT_H
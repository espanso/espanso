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

#ifndef ESPANSO_UI_H
#define ESPANSO_UI_H

#include <stdint.h>

// Explicitly define this constant as we need to use it from the Rust side
// https://docs.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation
#define MAX_FILE_PATH 260
#define MAX_ICON_COUNT 3

#define UI_EVENT_TYPE_ICON_CLICK 1
#define UI_EVENT_TYPE_CONTEXT_MENU_CLICK 2

typedef struct {
  int32_t show_icon;

  wchar_t icon_paths[MAX_ICON_COUNT][MAX_FILE_PATH];
  int32_t icon_paths_count;
  wchar_t notification_icon_path[MAX_FILE_PATH];
} UIOptions;

typedef struct {
  int32_t event_type;
  uint32_t context_menu_id;
} UIEvent;

typedef void (*EventCallback)(void * self, UIEvent data);

// Initialize the hidden UI window, the tray icon and returns the window handle.
extern "C" void * ui_initialize(void * self, UIOptions options, int32_t * error_code);

// Run the event loop. Blocking call.
extern "C" int32_t ui_eventloop(void * window, EventCallback callback);

// Destroy the given window.
extern "C" int32_t ui_destroy(void * window);

// Send a termination event that exits the event loop
extern "C" void ui_exit(void * window);

// Updates the tray icon to the given one. The method accepts an index that refers to
// the icon within the UIOptions.icon_paths array.
extern "C" void ui_update_tray_icon(void * window, int32_t index);

// Show a native Windows 10 notification
extern "C" int32_t ui_show_notification(void * window, wchar_t * message);

// Display the context menu on the tray icon.
// Payload is passed as JSON as given the complex structure, parsing
// this manually would have been complex.
extern "C" int32_t ui_show_context_menu(void * window, char * payload);

#endif //ESPANSO_UI_H
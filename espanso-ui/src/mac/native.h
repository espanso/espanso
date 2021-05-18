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
#define MAX_FILE_PATH 1024
#define MAX_ICON_COUNT 3

#define UI_EVENT_TYPE_ICON_CLICK 1
#define UI_EVENT_TYPE_CONTEXT_MENU_CLICK 2

typedef struct {
  int32_t show_icon;

  char icon_paths[MAX_ICON_COUNT][MAX_FILE_PATH];
  int32_t icon_paths_count;
} UIOptions;

typedef struct {
  int32_t event_type;
  uint32_t context_menu_id;
} UIEvent;

typedef void (*EventCallback)(void * self, UIEvent data);

typedef struct
{
  UIOptions options;

  // Rust interop
  void *rust_instance;
  EventCallback event_callback;
} UIVariables;

// Initialize the Application delegate.
extern "C" void ui_initialize(void * self, UIOptions options);

// Run the event loop. Blocking call.
extern "C" int32_t ui_eventloop(EventCallback callback);

// Stops the application eventloop.
extern "C" void ui_exit();

// Updates the tray icon to the given one. The method accepts an index that refers to
// the icon within the UIOptions.icon_paths array.
extern "C" void ui_update_tray_icon(int32_t index);

// Show a native notification
extern "C" void ui_show_notification(char * message, double delay);

// Display the context menu on the tray icon.
// Payload is passed as JSON as given the complex structure, parsing
// this manually would have been complex.
extern "C" void ui_show_context_menu(char * payload);

#endif //ESPANSO_UI_H
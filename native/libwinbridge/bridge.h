/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

#ifndef ESPANSO_BRIDGE_H
#define ESPANSO_BRIDGE_H

#include <stdio.h>
#include <stdint.h>

extern void * manager_instance;

/*
 * Initialize the Windows parameters
 * return: 1 if OK, -1 otherwise.
 */
extern "C" int32_t initialize(void * self, wchar_t * ico_path, wchar_t * bmp_path);

/*
 * Called when a new keypress is made, the first argument is an int array,
 * while the second is the size of the array.
 */
typedef void (*KeypressCallback)(void * self, int32_t *buffer, int32_t len, int32_t is_modifier, int32_t key_code);
extern KeypressCallback keypress_callback;

/*
 * Register the callback that will be called when a keypress was made
 */
extern "C" void register_keypress_callback(KeypressCallback callback);

/*
 * Start the event loop indefinitely. Blocking call.
 */
extern "C" void eventloop();

// Keyboard Manager

/*
 * Type the given string by simulating Key Presses
 */
extern "C" void send_string(const wchar_t * string);

/*
 * Send the given Virtual Key press
 */
extern "C" void send_vkey(int32_t vk);

/*
 * Send the backspace keypress, *count* times.
 */
extern "C" void delete_string(int32_t count);

// Detect current application commands

/*
 * Return the active windows's title
 */
extern "C" int32_t get_active_window_name(wchar_t * buffer, int32_t size);

/*
 * Return the active windows's executable path
 */
extern "C" int32_t get_active_window_executable(wchar_t * buffer, int32_t size);

// UI

/*
 * Called when the tray icon is clicked
 */
typedef void (*IconClickCallback)(void * self);
extern IconClickCallback icon_click_callback;
extern "C" void register_icon_click_callback(IconClickCallback callback);

// CONTEXT MENU

typedef struct {
    int32_t id;
    int32_t type;
    wchar_t name[100];
} MenuItem;

extern "C" int32_t show_context_menu(MenuItem * items, int32_t count);

/*
 * Called when the context menu is clicked
 */
typedef void (*ContextMenuClickCallback)(void * self, int32_t id);
extern ContextMenuClickCallback context_menu_click_callback;
extern "C" void register_context_menu_click_callback(ContextMenuClickCallback callback);

// NOTIFICATION

/*
 * Show a window containing the notification.
 */
extern "C" int32_t show_notification(wchar_t * message);

/*
 * Close the notification if present
 */
extern "C" void close_notification();


#endif //ESPANSO_BRIDGE_H
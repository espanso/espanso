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

#include <stdint.h>

extern "C" {

extern void * context_instance;
extern char * icon_path;

/*
* Initialize the AppDelegate and check for accessibility permissions
*/
int32_t initialize(void * context, const char * icon_path);

/*
 * Start the event loop indefinitely. Blocking call.
 */
int32_t eventloop();

/*
 * Initialize the application and start the headless eventloop, used for the espanso detect command
 */
int32_t headless_eventloop();

/*
 * Called when a new keypress is made, the first argument is an char array,
 * while the second is the size of the array.
 */
typedef void (*KeypressCallback)(void * self, const char *buffer, int32_t len, int32_t is_modifier, int32_t key_code);

extern KeypressCallback keypress_callback;

/*
 * Register the callback that will be called when a keypress was made
 */
void register_keypress_callback(KeypressCallback callback);

/*
 * Type the given string by using the CGEventKeyboardSetUnicodeString call
 */
void send_string(const char * string);

/*
 * Send the Virtual Key press
 */
void send_vkey(int32_t vk);

/*
 * Send the Virtual Key press multiple times
 */
void send_multi_vkey(int32_t vk, int32_t count);

/*
 * Send the backspace keypress, *count* times.
 */
void delete_string(int32_t count);

/*
 * Trigger normal paste ( Pressing CMD+V )
 */
void trigger_paste();

// UI

/*
 * Called when the tray icon is clicked
 */
typedef void (*IconClickCallback)(void * self);
extern IconClickCallback icon_click_callback;
void register_icon_click_callback(IconClickCallback callback);

// CONTEXT MENU

typedef struct {
    int32_t id;
    int32_t type;
    char name[100];
} MenuItem;

int32_t show_context_menu(MenuItem * items, int32_t count);

/*
 * Called when the context menu is clicked
 */
typedef void (*ContextMenuClickCallback)(void * self, int32_t id);
extern ContextMenuClickCallback context_menu_click_callback;
extern "C" void register_context_menu_click_callback(ContextMenuClickCallback callback);

// SYSTEM

/*
 * Check if espanso is authorized to control accessibility features, needed to detect key presses.
 * @return
 */
int32_t check_accessibility();

/*
 * Prompt to authorize the accessibility features.
 * @return
 */
int32_t prompt_accessibility();

/*
 * Open Security & Privacy settings panel
 * @return
 */
void open_settings_panel();

/*
 * Return the active NSRunningApplication path
 */
int32_t get_active_app_bundle(char * buffer, int32_t size);

/*
 * Return the active NSRunningApplication bundle identifier
 */
int32_t get_active_app_identifier(char * buffer, int32_t size);

// CLIPBOARD

/*
 * Return the clipboard text
 */
int32_t get_clipboard(char * buffer, int32_t size);

/*
 * Set the clipboard text
 */
int32_t set_clipboard(char * text);

/*
 * Set the clipboard image to the given file
 */
int32_t set_clipboard_image(char * path);


};
#endif //ESPANSO_BRIDGE_H

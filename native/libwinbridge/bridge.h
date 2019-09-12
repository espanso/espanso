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

// CONTEXT MENU

typedef struct {
    int32_t id;
    int32_t type;
    wchar_t name[100];
} MenuItem;

/*
 * Called when the context menu is created.
 */
typedef void (*MenuItemCallback)(void * self, MenuItem * items, int32_t * item_count);

extern MenuItemCallback menu_item_callback;
extern "C" void register_menu_item_callback(MenuItemCallback callback);

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
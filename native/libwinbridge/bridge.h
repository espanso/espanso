#ifndef ESPANSO_BRIDGE_H
#define ESPANSO_BRIDGE_H

#include <stdio.h>
#include <stdint.h>

/*
 * Called when a new keypress is made, the first argument is an int array,
 * while the second is the size of the array.
 */
typedef void (*KeypressCallback)(void * self, int32_t *buffer, int32_t len, int32_t is_modifier, int32_t key_code);

extern KeypressCallback keypress_callback;
extern void * interceptor_instance;

/*
 * Register the callback that will be called when a keypress was made
 */
extern "C" void register_keypress_callback(void *self, KeypressCallback callback);

/*
 * Initialize the Windows worker's parameters
 * return: 1 if OK, -1 otherwise.
 */
extern "C" int32_t initialize_window();

/*
 * Start the event loop indefinitely. Blocking call.
 */
extern "C" void eventloop();

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

/*
 * Return the active windows's title
 */
extern "C" int32_t get_active_window_name(wchar_t * buffer, int32_t size);

/*
 * Return the active windows's executable path
 */
extern "C" int32_t get_active_window_executable(wchar_t * buffer, int32_t size);

#endif //ESPANSO_BRIDGE_H
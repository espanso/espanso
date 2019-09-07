#ifndef ESPANSO_BRIDGE_H
#define ESPANSO_BRIDGE_H

#include <stdint.h>

/*
 * Initialize the X11 context and parameters
 */
extern "C" int32_t initialize();

/*
 * Start the event loop indefinitely. Blocking call.
 */
extern "C" int32_t eventloop();

/*
 * Clean all the X11 resources allocated during the initialization.
 */
extern "C" void cleanup();

/*
 * Called when a new keypress is made, the first argument is an char array,
 * while the second is the size of the array.
 */
typedef void (*KeypressCallback)(void * self, const char *buffer, int32_t len, int32_t is_modifier, int32_t key_code);

extern KeypressCallback keypress_callback;
extern void * interceptor_instance;

/*
 * Register the callback that will be called when a keypress was made
 */
extern "C" void register_keypress_callback(void *self, KeypressCallback callback);

/*
 * Type the given string by simulating Key Presses
 */
extern "C" void send_string(const char * string);

/*
 * Send the backspace keypress, *count* times.
 */
extern "C" void delete_string(int32_t count);

/*
 * Trigger normal paste ( Pressing CTRL+V )
 */
extern "C" void trigger_paste();

/*
 * Trigger terminal paste ( Pressing CTRL+SHIFT+V )
 */
extern "C" void trigger_terminal_paste();


// SYSTEM MODULE

/*
 * Return the active windows's WM_NAME
 */
extern "C" int32_t get_active_window_name(char * buffer, int32_t size);

/*
 * Return the active windows's WM_CLASS
 */
extern "C" int32_t get_active_window_class(char * buffer, int32_t size);

#endif //ESPANSO_BRIDGE_H

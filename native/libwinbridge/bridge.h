#ifndef ESPANSO_BRIDGE_H
#define ESPANSO_BRIDGE_H

#include <stdio.h>
#include <stdint.h>

/*
 * Called when a new keypress is made, the first argument is an int array,
 * while the second is the size of the array.
 */
typedef void (*keypress_callback)(void * self, int32_t *buffer, int32_t len);

extern keypress_callback keypressCallback;
extern void * backend_instance;

/*
 * Register the callback that will be called when a keypress was made
 */
extern "C" void register_keypress_callback(void *self, keypress_callback callback);

/*
 * Initialize the Windows worker's parameters
 * return: 1 if OK, -1 otherwise.
 */
extern "C" int32_t initialize_window();

/*
 * Start the event loop indefinitely. Blocking call.
 */
extern "C" void eventloop();

#endif //ESPANSO_BRIDGE_H

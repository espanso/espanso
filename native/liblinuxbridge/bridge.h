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
typedef void (*KeypressCallback)(void * self, char *buffer, int32_t len);

extern KeypressCallback keypress_callback;
extern void * interceptor_instance;

/*
 * Register the callback that will be called when a keypress was made
 */
extern "C" void register_keypress_callback(void *self, KeypressCallback callback);

#endif //ESPANSO_BRIDGE_H

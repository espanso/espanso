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

#endif //ESPANSO_BRIDGE_H

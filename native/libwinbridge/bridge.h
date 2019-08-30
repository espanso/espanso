#ifndef ESPANSO_BRIDGE_H
#define ESPANSO_BRIDGE_H

#include <stdio.h>

/*
 * Initialize the Windows worker's parameters
 */
extern "C" void initialize();

/*
 * Start the event loop indefinitely. Blocking call.
 */
extern "C" void eventloop();

#endif //ESPANSO_BRIDGE_H

#ifndef ESPANSO_BRIDGE_H
#define ESPANSO_BRIDGE_H

#include <stdint.h>

extern "C" {

/*
* Initialize the AppDelegate and check for accessibility permissions
*/
int32_t initialize();

/*
 * Start the event loop indefinitely. Blocking call.
 */
int32_t eventloop();

/*
 * Called when a new keypress is made, the first argument is an char array,
 * while the second is the size of the array.
 */
typedef void (*KeypressCallback)(void * self, const char *buffer, int32_t len);

extern KeypressCallback keypress_callback;
extern void * interceptor_instance;

/*
 * Register the callback that will be called when a keypress was made
 */
void register_keypress_callback(void *self, KeypressCallback callback);

/*
 * Type the given string by using the CGEventKeyboardSetUnicodeString call
 */
void send_string(const char * string);

/*
 * Send the backspace keypress, *count* times.
 */
extern "C" void delete_string(int32_t count);

};



#endif //ESPANSO_BRIDGE_H

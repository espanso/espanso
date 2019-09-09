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
typedef void (*KeypressCallback)(void * self, const char *buffer, int32_t len, int32_t is_modifier, int32_t key_code);

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
 * Send the Virtual Key press
 */
void send_vkey(int32_t vk);

/*
 * Send the backspace keypress, *count* times.
 */
void delete_string(int32_t count);

// SYSTEM

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

};
#endif //ESPANSO_BRIDGE_H

//
// Most of this code has been taken from the wonderful XDOTOOL: https://github.com/jordansissel/xdotool/blob/master/COPYRIGHT
// and modified to use XSendEvent instead of XTestFakeKeyEvent.

#ifndef LIBLINUXBRIDGE_FAST_XDO_H
#define LIBLINUXBRIDGE_FAST_XDO_H

extern "C" {  // Needed to avoid C++ compiler name mangling
#include <xdo.h>
}

KeySym fast_keysym_from_char(const xdo_t *xdo, wchar_t key);
void fast_charcodemap_from_char(const xdo_t *xdo, charcodemap_t *key);
void fast_charcodemap_from_keysym(const xdo_t *xdo, charcodemap_t *key, KeySym keysym);
void fast_init_xkeyevent(const xdo_t *xdo, XKeyEvent *xk);
void fast_send_key(const xdo_t *xdo, Window window, charcodemap_t *key,
                          int modstate, int is_press, useconds_t delay);
int fast_enter_text_window(const xdo_t *xdo, Window window, const char *string, useconds_t delay);
void fast_send_event(const xdo_t *xdo, Window window, int keycode, int pressed);

#endif //LIBLINUXBRIDGE_FAST_XDO_H

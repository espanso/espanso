//
// Most of this code has been taken from the wonderful XDOTOOL: https://github.com/jordansissel/xdotool/blob/master/COPYRIGHT
// and modified to use XSendEvent instead of XTestFakeKeyEvent.

#include <locale.h>
#include <stdio.h>
#include <stdlib.h>

#include "fast_xdo.h"

extern "C" {  // Needed to avoid C++ compiler name mangling
#include <xdo.h>
}

void fast_init_xkeyevent(const xdo_t *xdo, XKeyEvent *xk) {
    xk->display = xdo->xdpy;
    xk->subwindow = None;
    xk->time = CurrentTime;
    xk->same_screen = True;

    /* Should we set these at all? */
    xk->x = xk->y = xk->x_root = xk->y_root = 1;
}

void fast_send_key(const xdo_t *xdo, Window window, charcodemap_t *key,
                   int modstate, int is_press, useconds_t delay) {
    /* Properly ensure the modstate is set by finding a key
     * that activates each bit in the modifier state */
    int mask = modstate | key->modmask;

    /* Since key events have 'state' (shift, etc) in the event, we don't
         * need to worry about key press ordering. */
    XKeyEvent xk;
    fast_init_xkeyevent(xdo, &xk);
    xk.window = window;
    xk.keycode = key->code;
    xk.state = mask | (key->group << 13);
    xk.type = (is_press ? KeyPress : KeyRelease);
    XSendEvent(xdo->xdpy, xk.window, True, 0, (XEvent *)&xk);

    /* Skipping the usleep if delay is 0 is much faster than calling usleep(0) */
    XFlush(xdo->xdpy);
    if (delay > 0) {
        usleep(delay);
    }
}

int fast_send_keysequence_window_list_do(const xdo_t *xdo, Window window, charcodemap_t *keys,
                                        int nkeys, int pressed, int *modifier, useconds_t delay) {
    int i = 0;
    int modstate = 0;
    int keymapchanged = 0;

    /* Find an unused keycode in case we need to bind unmapped keysyms */
    KeySym *keysyms = NULL;
    int keysyms_per_keycode = 0;
    int scratch_keycode = 0; /* Scratch space for temporary keycode bindings */
    keysyms = XGetKeyboardMapping(xdo->xdpy, xdo->keycode_low,
                                  xdo->keycode_high - xdo->keycode_low,
                                  &keysyms_per_keycode);

    /* Find a keycode that is unused for scratchspace */
    for (i = xdo->keycode_low; i <= xdo->keycode_high; i++) {
        int j = 0;
        int key_is_empty = 1;
        for (j = 0; j < keysyms_per_keycode; j++) {
            /*char *symname;*/
            int symindex = (i - xdo->keycode_low) * keysyms_per_keycode + j;
            /*symname = XKeysymToString(keysyms[symindex]);*/
            if (keysyms[symindex] != 0) {
                key_is_empty = 0;
            } else {
                break;
            }
        }
        if (key_is_empty) {
            scratch_keycode = i;
            break;
        }
    }
    XFree(keysyms);

    /* Allow passing NULL for modifier in case we don't care about knowing
     * the modifier map state after we finish */
    if (modifier == NULL)
        modifier = &modstate;

    for (i = 0; i < nkeys; i++) {
        if (keys[i].needs_binding == 1) {
            KeySym keysym_list[] = { keys[i].symbol };
            //_xdo_debug(xdo, "Mapping sym %lu to %d", keys[i].symbol, scratch_keycode);
            XChangeKeyboardMapping(xdo->xdpy, scratch_keycode, 1, keysym_list, 1);
            XSync(xdo->xdpy, False);
            /* override the code in our current key to use the scratch_keycode */
            keys[i].code = scratch_keycode;
            keymapchanged = 1;
        }

        //fprintf(stderr, "keyseqlist_do: Sending %lc %s (%d, mods %x)\n",
        //keys[i].key, (pressed ? "down" : "up"), keys[i].code, *modifier);
        fast_send_key(xdo, window, &(keys[i]), *modifier, pressed, delay);

        if (keys[i].needs_binding == 1) {
            /* If we needed to make a new keymapping for this keystroke, we
             * should sync with the server now, after the keypress, so that
             * the next mapping or removal doesn't conflict. */
            XSync(xdo->xdpy, False);
        }

        if (pressed) {
            *modifier |= keys[i].modmask;
        } else {
            *modifier &= ~(keys[i].modmask);
        }
    }


    if (keymapchanged) {
        KeySym keysym_list[] = { 0 };
        //printf(xdo, "Reverting scratch keycode (sym %lu to %d)",
        //           keys[i].symbol, scratch_keycode);
        XChangeKeyboardMapping(xdo->xdpy, scratch_keycode, 1, keysym_list, 1);
    }

    /* Necessary? */
    XFlush(xdo->xdpy);
    return XDO_SUCCESS;
}

KeySym fast_keysym_from_char(const xdo_t *xdo, wchar_t key) {
    int i = 0;
    int len = xdo->charcodes_len;

    //printf("Finding symbol for key '%c'\n", key);
    for (i = 0; i < len; i++) {
        //printf("  => %c vs %c (%d)\n",
        //key, xdo->charcodes[i].key, (xdo->charcodes[i].key == key));
        if (xdo->charcodes[i].key == key) {
            //printf("  => MATCH to symbol: %lu\n", xdo->charcodes[i].symbol);
            return xdo->charcodes[i].symbol;
        }
    }

    if (key >= 0x100) key += 0x01000000;
    if (XKeysymToString(key)) return key;
    return NoSymbol;
}

void fast_charcodemap_from_keysym(const xdo_t *xdo, charcodemap_t *key, KeySym keysym) {
    int i = 0;
    int len = xdo->charcodes_len;

    key->code = 0;
    key->symbol = keysym;
    key->group = 0;
    key->modmask = 0;
    key->needs_binding = 1;

    for (i = 0; i < len; i++) {
        if (xdo->charcodes[i].symbol == keysym) {
            key->code = xdo->charcodes[i].code;
            key->group = xdo->charcodes[i].group;
            key->modmask = xdo->charcodes[i].modmask;
            key->needs_binding = 0;
            return;
        }
    }
}

void fast_charcodemap_from_char(const xdo_t *xdo, charcodemap_t *key) {
    KeySym keysym = fast_keysym_from_char(xdo, key->key);
    fast_charcodemap_from_keysym(xdo, key, keysym);
}

/* XXX: Return proper code if errors found */
int fast_enter_text_window(const xdo_t *xdo, Window window, const char *string, useconds_t delay) {

    /* Since we're doing down/up, the delay should be based on the number
     * of keys pressed (including shift). Since up/down is two calls,
     * divide by two. */
    delay /= 2;

    /* XXX: Add error handling */
    //int nkeys = strlen(string);
    //charcodemap_t *keys = calloc(nkeys, sizeof(charcodemap_t));
    charcodemap_t key;
    //int modifier = 0;
    setlocale(LC_CTYPE,"");
    mbstate_t ps = { 0 };
    ssize_t len;
    while ( (len = mbsrtowcs(&key.key, &string, 1, &ps)) ) {
        if (len == -1) {
            fprintf(stderr, "Invalid multi-byte sequence encountered\n");
            return XDO_ERROR;
        }
        fast_charcodemap_from_char(xdo, &key);
        if (key.code == 0 && key.symbol == NoSymbol) {
            fprintf(stderr, "I don't what key produces '%lc', skipping.\n",
                    key.key);
            continue;
        } else {
            //printf("Found key for %c\n", key.key);
            //printf("code: %d\n", key.code);
            //printf("sym: %s\n", XKeysymToString(key.symbol));
        }

        //printf(stderr,
        //"Key '%c' maps to code %d / sym %lu in group %d / mods %d (%s)\n",
        //key.key, key.code, key.symbol, key.group, key.modmask,
        //(key.needs_binding == 1) ? "needs binding" : "ok");

        //_xdo_send_key(xdo, window, keycode, modstate, True, delay);
        //_xdo_send_key(xdo, window, keycode, modstate, False, delay);
        fast_send_keysequence_window_list_do(xdo, window, &key, 1, True, NULL, delay / 2);
        key.needs_binding = 0;
        fast_send_keysequence_window_list_do(xdo, window, &key, 1, False, NULL, delay / 2);

        /* XXX: Flush here or at the end? or never? */
        //XFlush(xdo->xdpy);
    } /* walk string generating a keysequence */

    //free(keys);
    return XDO_SUCCESS;
}

void fast_send_event(const xdo_t *xdo, Window window, int keycode, int pressed) {
    XKeyEvent xk;
    xk.display = xdo->xdpy;
    xk.window = window;
    xk.root = XDefaultRootWindow(xdo->xdpy);
    xk.subwindow = None;
    xk.time = CurrentTime;
    xk.x = 1;
    xk.y = 1;
    xk.x_root = 1;
    xk.y_root = 1;
    xk.same_screen = True;
    xk.keycode = keycode;
    xk.state = 0;
    xk.type = (pressed ? KeyPress : KeyRelease);

    XEvent event;
    event.xkey =xk;

    XSendEvent(xdo->xdpy, window, True, 0, &event);
}

To support EVDEV injection

At startup, the EVDEVInjector has to populate a lookup table to find the string <-> keycode + modifiers pairs.

* We know that the keycode goes from 1 to 256
* We can then rely on the `xkb_state_key_get_utf8` to find the string correspondent to each keycode
* Then we cycle between every modifier combination, updating the `xkb_state` with `xkb_state_update_key`

Ref: https://xkbcommon.org/doc/current/structxkb__keymap.html

```
  1 #include <xkbcommon/xkbcommon.h>
  2 #include <stdio.h>
  3 
  4 int main() {
  5   struct xkb_context *ctx = xkb_context_new(XKB_CONTEXT_NO_FLAGS);
  6   struct xkb_keymap *keymap =xkb_keymap_new_from_names(ctx, NULL, 0);
  7   struct xkb_state *state = xkb_state_new(keymap);
  8   // a = 38
  9 
 10   xkb_state_update_key(state, 42 + 8, XKB_KEY_DOWN);
 11 
 12   xkb_layout_index_t num = xkb_keymap_num_layouts_for_key(keymap, 42 + 8);
 13   char buff[10];
 14   xkb_state_key_get_utf8(state, 38, buff, 9);
 15 
 16   printf("hey %s %d\n", buff, num);
 17 }
```

The available modifiers can be found in the https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h

#define KEY_LEFTCTRL		29
#define KEY_LEFTSHIFT		42
#define KEY_RIGHTSHIFT		54
#define KEY_LEFTALT		56
#define KEY_LEFTMETA		125
#define KEY_RIGHTMETA		126
#define KEY_RIGHTCTRL		97
#define KEY_RIGHTALT		100

All these codes have to be added the EVDEV_OFFSET = 8

From the lookup table, we can generate the input event as shown here: https://github.com/ReimuNotMoe/ydotool/blob/7972e5e3390489c1395b06ca9dc7639763c7cc98/Tools/Type/Type.cpp

Note that we also need to inject the correct modifiers to obtain the text
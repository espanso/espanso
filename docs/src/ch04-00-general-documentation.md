# General documentation

Becase we are just at the beginning of a genereral dev documentation, this
chapters will sound a lot untidy.

## General great resources to learn

- We use `gdiplus` in some places. [Here is the doc from mslearn](https://learn.microsoft.com/en-us/windows/win32/gdiplus/-gdiplus-gdi-start)

## Cache folder

Espanso saves the following files in the cache folder (for example in macos
`/Users/username/Library/Caches`)

- `kvs/` the folder of `espanso-kvs`
  inside it has
  - `has_completed_wizard`
  - `has_displayed_welcome`
  - `has_selected_auto_start_option`

  which have true inside those files.

- some pictures to ease the loading of the app (
  - accessibility_1.png,
  - accessibility_2.png,
  - disabledv2.png
  - icon_explain_image.png
  - icon_no_background.png
  - iconv2.png
  - normalv2.png
  - searchv2.png
  - systemdisabledv2.png

- the log `espanso.log`
- the locks
  - `espanso.lock` 
  - the worker `espanso-worker.lock`
  - and the daemon `espanso-daemon.lock`

## Vendored images

We have a couple of vendored images

- `../../scripts/vendor-app-image/linuxdeploy-x86_64.AppImage` and
- `../../scripts/vendor-app-image/appimagetool-x86_64.AppImage`

are vendored build tools here to have reproducible builds, otherwise
we might run into these issues again:

[github Issue](https://github.com/espanso/espanso/issues/900)

- `../../espanso-modulo/vendor/wxWidgets-3.1.5.zip` is the built image
from [wxWidgets webpage](https://wxwidgets.org/) and 
[github page](https://github.com/wxWidgets/wxWidgets)



## espanso-inject

### EVDEV injection (`../../espanso-inject/src/evdev/`)
At startup, the EVDEVInjector has to populate a lookup table to find the string <-> keycode + modifiers pairs.

* We know that the keycode goes from 1 to 256
* We can then rely on the `xkb_state_key_get_utf8` to find the string correspondent to each keycode
* Then we cycle between every modifier combination, updating the `xkb_state` with `xkb_state_update_key`

Ref: https://xkbcommon.org/doc/current/structxkb__keymap.html

```c
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

```c
#define KEY_LEFTCTRL		29
#define KEY_LEFTSHIFT		42
#define KEY_RIGHTSHIFT		54
#define KEY_LEFTALT		56
#define KEY_LEFTMETA		125
#define KEY_RIGHTMETA		126
#define KEY_RIGHTCTRL		97
#define KEY_RIGHTALT		100
#define KEY_CAPSLOCK		58
#define KEY_NUMLOCK		69
```

All these codes have to be added the `EVDEV_OFFSET = 8`

From the lookup table, we can generate the input event as shown here: 
[ydotool `Type.cpp`](https://github.com/ReimuNotMoe/ydotool/blob/7972e5e3390489c1395b06ca9dc7639763c7cc98/Tools/Type/Type.cpp)

Note that we also need to inject the correct modifiers to obtain the text

### xdotool (`../../espanso-inject/src/x11/xdotool/vendor/`)

We also have vendored code in the `../../espanso-inject/src/x11/xdotool/vendor`
folder, to have a fallback injection module that relies on the awesome `xdotool`
project. [Github page](https://github.com/jordansissel/xdotool)

## espanso-detect 

### Detect EVDEV (`../../espanso-detect/src/evdev/`)

This module is used to detect keyboard and mouse input using EVDEV layer, which is necessary on Wayland.
The module started as a port of this `xkbcommon` example

https://github.com/xkbcommon/libxkbcommon/blob/master/tools/interactive-evdev.c

## espanso-clipboard

### Notes on Wayland and clipboard support (`../../espanso-clipboard/src/wayland/`)

#### Running espanso as another user

When running espanso as another user, we need to set up a couple of permissions
in order to enable the clipboard tools to correctly connect to the Wayland desktop.

In particular, we need to add the `espanso` user to the same group as the current user
so that it can access the `/run/user/X` directory (with X depending on the user).

```bash
# Find the current user wayland dir with
echo $XDG_RUNTIME_DIR  # in my case output: /run/user/1000

ls -la /run/user/1000

# Now add the `espanso` user to the current user group
sudo usermod -a -G freddy espanso

# Give permissions to the group
chmod g+rwx /run/user/1000

# Give write permission to the wayland socket
chmod g+w /run/user/1000/wayland-0
```

Now the clipboard should work as expected

#### Or, a better implementation

On some Wayland compositors (currently sway), the "wlr-data-control" protocol
could enable the use of a much more efficient implementation relying on the
"wl-clipboard-rs" crate.

[`wl-clipboard-rs` Github link](https://github.com/YaLTeR/wl-clipboard-rs/issues/8)

### X11NativeClipboard (`../../espanso-clipboard/src/x11/native/`)

The X11NativeClipboard modules uses the wonderful [clip](https://github.com/dacap/clip) library
by David Capello to manipulate the clipboard.

At the time of writing, the library is MIT licensed.


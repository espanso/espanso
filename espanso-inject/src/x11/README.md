Same approach as evdev, but the lookup logic is:

#include <locale.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <X11/Xlibint.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/cursorfont.h>
#include <X11/keysymdef.h>
#include <X11/keysym.h>
#include <X11/extensions/record.h>
#include <X11/extensions/XTest.h>
#include <X11/XKBlib.h>
#include <X11/Xatom.h>

Display *data_disp = NULL;

int main() {
    data_disp = XOpenDisplay(NULL);

    for (int code = 0; code<256; code++) {
        for (int state = 0; state < 256; state++) {
        XKeyEvent event;
    event.display     = data_disp;
    event.window      = XDefaultRootWindow(data_disp);
    event.root        = XDefaultRootWindow(data_disp);
    event.subwindow   = None;
    event.time        = 0;
    event.x           = 1;
    event.y           = 1;
    event.x_root      = 1;
    event.y_root      = 1;
    event.same_screen = True;
    event.keycode     = code + 8;
    event.state       = state;
    event.type = KeyPress;

    char buffer[10];
    int res = XLookupString(&event, buffer, 9, NULL, NULL);

    printf("hey %d %d %s\n", code, state, buffer);
    }
    
    }
    
}


This way, we get the state mask associated with a character, and we can pass it directly when injecting a character:
https://github.com/federico-terzi/espanso/blob/master/native/liblinuxbridge/fast_xdo.cpp#L37
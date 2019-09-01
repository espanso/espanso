#include "bridge.h"

#include <locale.h>
#include <stdio.h>
#include <stdlib.h>
#include <array>
#include <X11/Xlibint.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/cursorfont.h>
#include <X11/keysymdef.h>
#include <X11/keysym.h>
#include <X11/extensions/record.h>
#include <X11/extensions/XTest.h>
#include <X11/XKBlib.h>
extern "C" {  // Needed to avoid C++ compiler name mangling
    #include <xdo.h>
}

/*
This code uses the X11 Record Extension to receive keyboard
events. Documentation of this library can be found here:
https://www.x.org/releases/X11R7.6/doc/libXtst/recordlib.html

We will refer to this extension as RE from now on.
*/

/*
This struct is needed to receive events from the RE.
The funny thing is: it's not defined there, it should though.
The only place this is mentioned is the libxnee library,
so check that out if you need a reference.
*/
typedef union {
    unsigned char    type ;
    xEvent           event ;
    xResourceReq     req   ;
    xGenericReply    reply ;
    xError           error ;
    xConnSetupPrefix setup;
} XRecordDatum;

/*
Connections to the X server, RE recommends 2 connections:
one for recording control and one for reading the recorded data.
*/
Display *data_disp = NULL;
Display *ctrl_disp = NULL;

XRecordRange  *record_range;
XRecordContext context;

xdo_t * xdo_context;

// Callback invoked when a new key event occur.
void event_callback (XPointer, XRecordInterceptData*);

KeypressCallback keypress_callback;
void * interceptor_instance;

void register_keypress_callback(void * self, KeypressCallback callback) {
    keypress_callback = callback;
    interceptor_instance = self;
}


int32_t initialize() {
    setlocale(LC_ALL, "");

    /*
    Open the connections to the X server.
    RE recommends to open 2 connections to the X server:
    one for the recording control and one to read the protocol
    data.
    */
    ctrl_disp = XOpenDisplay(NULL);
    data_disp = XOpenDisplay(NULL);

    if (!ctrl_disp || !data_disp) {  // Display error
        return -1;
    }

    /*
    We must set the ctrl_disp to sync mode, or, when we the enable
    context in data_disp, there will be a fatal X error.
    */
    XSynchronize(ctrl_disp, True);

    int dummy;

    // Make sure the X RE is installed in this system.
    if (!XRecordQueryVersion(ctrl_disp, &dummy, &dummy)) {
        return -2;
    }

    // Make sure the X Keyboard Extension is installed
    if (!XkbQueryExtension(ctrl_disp, &dummy, &dummy, &dummy, &dummy, &dummy)) {
        return -3;
    }

    // Initialize the record range, that is the kind of events we want to track.
    record_range = XRecordAllocRange ();
    if (!record_range) {
        return -4;
    }
    record_range->device_events.first = KeyPress;
    record_range->device_events.last = KeyRelease;

    // We want to get the keys from all clients
    XRecordClientSpec  client_spec;
    client_spec = XRecordAllClients;

    // Initialize the context
    context = XRecordCreateContext(ctrl_disp, 0, &client_spec, 1, &record_range, 1);
    if (!context) {
        return -5;
    }

    xdo_context = xdo_new(NULL);
}

int32_t eventloop() {
    if (!XRecordEnableContext (data_disp, context, event_callback, NULL)) {
        return -1;
    }

    return 1;
}

void cleanup() {
    XRecordDisableContext(ctrl_disp, context);
    XRecordFreeContext(ctrl_disp, context);
    XFree (record_range);
    XCloseDisplay(data_disp);
    XCloseDisplay(ctrl_disp);
    xdo_free(xdo_context);
}

void event_callback(XPointer p, XRecordInterceptData *hook)
{
    // Make sure the event comes from the X11 server
    if (hook->category != XRecordFromServer) {
        XRecordFreeData(hook);
        return;
    }

    // Cast the event payload to a XRecordDatum, needed later to access the fields
    // This struct was hard to find and understand. Turn's out that all the
    // required data are included in the "event" field of this structure.
    // The funny thing is that it's not a XEvent as one might expect,
    // but a xEvent, a very different beast defined in the Xproto.h header.
    // I suggest you to look at that header if you want to understand where the
    // upcoming field where taken from.
    XRecordDatum *data = (XRecordDatum*) hook->data;

    int event_type = data->type;
    int key_code = data->event.u.u.detail;

    // In order to convert the key_code into the corresponding string,
    // we need to synthesize an artificial XKeyEvent, to feed later to the
    // XLookupString function.
    XKeyEvent event;
    event.display     = ctrl_disp;
    event.window      = data->event.u.focus.window;
    event.root        = XDefaultRootWindow(ctrl_disp);
    event.subwindow   = None;
    event.time        = data->event.u.keyButtonPointer.time;
    event.x           = 1;
    event.y           = 1;
    event.x_root      = 1;
    event.y_root      = 1;
    event.same_screen = True;
    event.keycode     = key_code;
    event.state       = data->event.u.keyButtonPointer.state;
    event.type = KeyPress;

    // Extract the corresponding chars.
    std::array<char, 10> buffer;
    int res = XLookupString(&event, buffer.data(), buffer.size(), NULL, NULL);

    switch (event_type) {
        case KeyRelease:
            //printf ("%d %d KeyPress: \t%s\t%s\n", key_code, res, XKeysymToString(XkbKeycodeToKeysym(ctrl_disp, key_code, 0, 0)), buffer.data());
            if (res > 0) {  // Send only printable chars, todo: change
                keypress_callback(interceptor_instance, buffer.data(), buffer.size());
            }
            break;
//        case KeyPress:
//            printf ("%d %d KeyPress: \t%s\t%s\t%d\n", keycode, res, XKeysymToString(XkbKeycodeToKeysym(ctrl_disp, keycode, 0, 0)), buff, buff[0]);
//            break;
        default:
            break;
    }

    XRecordFreeData(hook);
}

void send_string(const char * string) {
    xdo_enter_text_window(xdo_context, CURRENTWINDOW, string, 12000);
}

void delete_string(int32_t count) {
    for (int i = 0; i<count; i++) {
        xdo_send_keysequence_window(xdo_context, CURRENTWINDOW, "BackSpace", 8000);
    }
}
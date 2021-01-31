/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

/*
This code uses the X11 Record Extension to receive keyboard
events. Documentation of this library can be found here:
https://www.x.org/releases/X11R7.6/doc/libXtst/recordlib.html
We will refer to this extension as RE from now on.
*/

#include "native.h"

#include <locale.h>
#include <stdio.h>
#include <stdlib.h>
#include <array>
#include <string.h>
#include <memory>

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

/*
This struct is needed to receive events from the RE.
The funny thing is: it's not defined there, it should though.
The only place this is mentioned is the libxnee library,
so check that out if you need a reference.
*/
typedef union
{
  unsigned char type;
  xEvent event;
  xResourceReq req;
  xGenericReply reply;
  xError error;
  xConnSetupPrefix setup;
} XRecordDatum;

typedef struct
{
  // Connections to the X server, RE recommends 2 connections:
  // one for recording control and one for reading the recorded data.
  Display *data_disp;
  Display *ctrl_disp;
  XRecordRange *record_range;
  XRecordContext x_context;

  void *rust_instance;
  EventCallback event_callback;
} DetectContext;

void detect_event_callback(XPointer, XRecordInterceptData *);
int detect_error_callback(Display *display, XErrorEvent *error);

int32_t detect_check_x11()
{
  Display *check_disp = XOpenDisplay(NULL);

  if (!check_disp)
  {
    return 0;
  }

  XCloseDisplay(check_disp);
  return 1;
}

void *detect_initialize(void *_rust_instance, int32_t *error_code)
{
  setlocale(LC_ALL, "");

  std::unique_ptr<DetectContext> context;
  context.reset(new DetectContext());
  context->rust_instance = _rust_instance;

  // Open the connections to the X server.
  // RE recommends to open 2 connections to the X server:
  // one for the recording control and one to read the protocol
  // data.
  context->ctrl_disp = XOpenDisplay(NULL);
  context->data_disp = XOpenDisplay(NULL);

  if (!context->ctrl_disp || !context->data_disp)
  { // Display error
    *error_code = -1;
    return nullptr;
  }

  // We must set the ctrl_disp to sync mode, or, when we the enable
  // context in data_disp, there will be a fatal X error.
  XSynchronize(context->ctrl_disp, True);

  int dummy;

  // Make sure the X RE is installed in this system.
  if (!XRecordQueryVersion(context->ctrl_disp, &dummy, &dummy))
  {
    *error_code = -2;
    return nullptr;
  }

  // Make sure the X Keyboard Extension is installed
  if (!XkbQueryExtension(context->ctrl_disp, &dummy, &dummy, &dummy, &dummy, &dummy))
  {
    *error_code = -3;
    return nullptr;
  }

  // Initialize the record range, that is the kind of events we want to track.
  context->record_range = XRecordAllocRange();
  if (!context->record_range)
  {
    *error_code = -4;
    return nullptr;
  }
  context->record_range->device_events.first = KeyPress;
  context->record_range->device_events.last = ButtonRelease;

  // We want to get the keys from all clients
  XRecordClientSpec client_spec;
  client_spec = XRecordAllClients;

  // Initialize the context
  context->x_context = XRecordCreateContext(context->ctrl_disp, 0, &client_spec, 1, &context->record_range, 1);
  if (!context->x_context)
  {
    *error_code = -5;
    return nullptr;
  }

  if (!XRecordEnableContextAsync(context->data_disp, context->x_context, detect_event_callback, (XPointer)context.get()))
  {
    *error_code = -6;
    return nullptr;
  }

  // Setup a custom error handler
  XSetErrorHandler(&detect_error_callback);

  // Note: We might never get a MappingNotify event if the
  // modifier and keymap information was never cached in Xlib.
  // The next line makes sure that this happens initially.
  XKeysymToKeycode(context->ctrl_disp, XK_F1);

  // Release the unique_ptr to avoid freeing the context right-away.
  return context.release();
}

int32_t detect_eventloop(void *_context, EventCallback _callback)
{
  DetectContext *context = (DetectContext *)_context;
  if (!context)
  {
    return -1;
  }
  context->event_callback = _callback;

  bool running = true;

  int ctrl_fd = XConnectionNumber(context->ctrl_disp);
  int data_fd = XConnectionNumber(context->data_disp);

  while (running)
  {
    fd_set fds;
    FD_ZERO(&fds);
    FD_SET(ctrl_fd, &fds);
    FD_SET(data_fd, &fds);
    timeval timeout;
    timeout.tv_sec = 2;
    timeout.tv_usec = 0;
    select(max(ctrl_fd, data_fd) + 1,
                        &fds, NULL, NULL, &timeout);

    if (FD_ISSET(data_fd, &fds))
    {
      XRecordProcessReplies(context->data_disp);
    }
    if (FD_ISSET(ctrl_fd, &fds))
    {
      XEvent event;
      XNextEvent(context->ctrl_disp, &event);
      if (event.type == MappingNotify)
      {
        XMappingEvent *e = (XMappingEvent *)&event;
        if (e->request == MappingKeyboard)
        {
          XRefreshKeyboardMapping(e);
        }
      }
    }
  }

  return 1;
}

int32_t detect_destroy(void *_context)
{
  DetectContext *context = (DetectContext *)_context;
  if (!context)
  {
    return -1;
  }

  XRecordDisableContext(context->ctrl_disp, context->x_context);
  XRecordFreeContext(context->ctrl_disp, context->x_context);
  XFree(context->record_range);
  XCloseDisplay(context->data_disp);
  XCloseDisplay(context->ctrl_disp);
  delete context;

  return 1;
}

void detect_event_callback(XPointer p, XRecordInterceptData *hook)
{
  DetectContext *context = (DetectContext *)p;
  if (!context)
  {
    return;
  }

  // Make sure the event comes from the X11 server
  if (hook->category != XRecordFromServer)
  {
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
  XRecordDatum *data = (XRecordDatum *)hook->data;

  int event_type = data->type;
  int key_code = data->event.u.u.detail;

  // In order to convert the key_code into the corresponding string,
  // we need to synthesize an artificial XKeyEvent, to feed later to the
  // XLookupString function.
  XKeyEvent raw_event;
  raw_event.display = context->ctrl_disp;
  raw_event.window = data->event.u.focus.window;
  raw_event.root = XDefaultRootWindow(context->ctrl_disp);
  raw_event.subwindow = None;
  raw_event.time = data->event.u.keyButtonPointer.time;
  raw_event.x = 1;
  raw_event.y = 1;
  raw_event.x_root = 1;
  raw_event.y_root = 1;
  raw_event.same_screen = True;
  raw_event.keycode = key_code;
  raw_event.state = data->event.u.keyButtonPointer.state;
  raw_event.type = event_type;

  InputEvent event = {};

  // Extract the corresponding chars.
  int res = XLookupString(&raw_event, event.buffer, sizeof(event.buffer) - 1, NULL, NULL);
  if (res > 0)
  {
    event.buffer_len = res;
  }
  else
  {
    memset(event.buffer, 0, sizeof(event.buffer));
    event.buffer_len = 0;
  }
  KeySym key_sym = XLookupKeysym(&raw_event, 0);

  switch (event_type)
  {
  case KeyPress:
  {
    event.event_type = INPUT_EVENT_TYPE_KEYBOARD;
    event.key_code = key_code;
    event.key_sym = key_sym;
    event.status = INPUT_STATUS_PRESSED;
    break;
  }
  case KeyRelease:
  {
    event.event_type = INPUT_EVENT_TYPE_KEYBOARD;
    event.key_code = key_code;
    event.key_sym = key_sym;
    event.status = INPUT_STATUS_RELEASED;
    break;
  }
  case ButtonPress:
  {
    event.event_type = INPUT_EVENT_TYPE_MOUSE;
    event.key_code = key_code;
    event.status = INPUT_STATUS_PRESSED;
    break;
  }
  case ButtonRelease:
  {
    event.event_type = INPUT_EVENT_TYPE_MOUSE;
    event.key_code = key_code;
    event.status = INPUT_STATUS_RELEASED;
    break;
  }
  }

  if (event.event_type != 0 && context->event_callback)
  {
    context->event_callback(context->rust_instance, event);
  }

  XRecordFreeData(hook);
}

int detect_error_callback(Display *, XErrorEvent *error)
{
  fprintf(stderr, "X11 Reported an error, code: %d, request_code: %d, minor_code: %d\n", error->error_code, error->request_code, error->minor_code);
  return 0;
}
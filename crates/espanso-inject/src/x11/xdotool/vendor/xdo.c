/* xdo library
 * - getwindowfocus contributed by Lee Pumphret
 * - keysequence_{up,down} contributed by Magnus Boman
 *
 * See the following url for an explanation of how keymaps work in X11
 * http://www.in-ulm.de/~mascheck/X11/xmodmap.html
 */

#ifndef _XOPEN_SOURCE
#define _XOPEN_SOURCE 500
#endif /* _XOPEN_SOURCE */

#include <sys/select.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <strings.h>
#include <unistd.h>
#include <regex.h>
#include <ctype.h>
#include <locale.h>
#include <stdarg.h>

#include <X11/Xlib.h>
#include <X11/XKBlib.h>
#include <X11/Xatom.h>
#include <X11/Xresource.h>
#include <X11/Xutil.h>
#include <X11/extensions/XTest.h>
#include <X11/keysym.h>
#include <X11/cursorfont.h>

#include <xkbcommon/xkbcommon.h>

#include "xdo.h"
#include "xdo_util.h"

#define DEFAULT_DELAY 12

/**
 * The number of tries to check for a wait condition before aborting.
 * TODO(sissel): Make this tunable at runtime?
 */
#define MAX_TRIES 500

static void _xdo_populate_charcode_map(xdo_t *xdo);
static int _xdo_has_xtest(const xdo_t *xdo);

static KeySym _xdo_keysym_from_char(const xdo_t *xdo, wchar_t key);
static void _xdo_charcodemap_from_char(const xdo_t *xdo, charcodemap_t *key);
static void _xdo_charcodemap_from_keysym(const xdo_t *xdo, charcodemap_t *key, KeySym keysym);
//static int _xdo_get_shiftcode_if_needed(const xdo_t *xdo, char key);

static int _xdo_send_keysequence_window_to_keycode_list(const xdo_t *xdo, const char *keyseq,
                                            charcodemap_t **keys, int *nkeys);
static int _xdo_send_keysequence_window_do(const xdo_t *xdo, Window window, const char *keyseq,
                               int pressed, int *modifier, useconds_t delay);
static int _xdo_ewmh_is_supported(const xdo_t *xdo, const char *feature);
static void _xdo_init_xkeyevent(const xdo_t *xdo, XKeyEvent *xk);
static void _xdo_send_key(const xdo_t *xdo, Window window, charcodemap_t *key,
                          int modstate, int is_press, useconds_t delay);
static void _xdo_send_modifier(const xdo_t *xdo, int modmask, int is_press);

static int _xdo_query_keycode_to_modifier(XModifierKeymap *modmap, KeyCode keycode);
static int _xdo_mousebutton(const xdo_t *xdo, Window window, int button, int is_press);

static int _is_success(const char *funcname, int code, const xdo_t *xdo);
static void _xdo_debug(const xdo_t *xdo, const char *format, ...);
static void _xdo_eprintf(const xdo_t *xdo, int hushable, const char *format, ...);

/* context-free functions */
static wchar_t _keysym_to_char(KeySym keysym);

/* Default to -1, initialize it when we need it */
static Atom atom_NET_WM_PID = -1;
static Atom atom_NET_WM_NAME = -1;
static Atom atom_WM_NAME = -1;
static Atom atom_STRING = -1;
static Atom atom_UTF8_STRING = -1;

xdo_t* xdo_new(const char *display_name) {
  Display *xdpy;

  if (display_name == NULL) {
    display_name = XDisplayName(display_name);
  }

#define DISPLAY_HINT "Is there an Xorg or other X server running? You can try setting 'export DISPLAY=:0' and trying again."
  if (display_name == NULL) {
    fprintf(stderr, "Error: No DISPLAY environment variable is set. " DISPLAY_HINT "\n");
    return NULL;
  }

  if (*display_name == '\0') {
    fprintf(stderr, "Error: DISPLAY environment variable is empty. " DISPLAY_HINT "\n");
    return NULL;
  }

  if ((xdpy = XOpenDisplay(display_name)) == NULL) {
    return NULL;
  }

  return xdo_new_with_opened_display(xdpy, display_name, 1);
}

xdo_t* xdo_new_with_opened_display(Display *xdpy, const char *display,
                                   int close_display_when_freed) {
  xdo_t *xdo = NULL;

  if (xdpy == NULL) {
    /* Can't use _xdo_eprintf yet ... */
    fprintf(stderr, "xdo_new: xdisplay I was given is a null pointer\n");
    return NULL;
  }

  // This library and xdotool do not work correctly on Wayland/XWayland.
  // Try to detect XWayland and warn the user about problems.
  // TODO(sissel): This was disabled due to issue #346
  //  -- xdotool works on XWayland for some operations, so it isn't helpful to refuse all usage on XWayland.
  //if (appears_to_be_wayland(xdpy)) {
    //fprintf(stderr, "The X server at %s appears to be XWayland. Unfortunately, XWayland does not correctly support the features used by libxdo and xdotool.\n", display);
    //return NULL;
  //}


  /* XXX: Check for NULL here */
  xdo = malloc(sizeof(xdo_t));
  memset(xdo, 0, sizeof(xdo_t));

  xdo->xdpy = xdpy;
  xdo->close_display_when_freed = close_display_when_freed;

  if (display == NULL) {
    display = "unknown";
  }

  if (getenv("XDO_QUIET")) {
    xdo->quiet = True;
  }

  if (_xdo_has_xtest(xdo)) {
    xdo_enable_feature(xdo, XDO_FEATURE_XTEST);
    _xdo_debug(xdo, "XTEST enabled.");
  } else {
    _xdo_eprintf(xdo, False, "Warning: XTEST extension unavailable on '%s'. Some"
                " functionality may be disabled; See 'man xdotool' for more"
                " info.", xdo->display_name);
    xdo_disable_feature(xdo, XDO_FEATURE_XTEST);
  }

  _xdo_populate_charcode_map(xdo);
  return xdo;
}

void xdo_free(xdo_t *xdo) {
  if (xdo == NULL)
    return;

  free(xdo->display_name);
  free(xdo->charcodes);
  if (xdo->xdpy && xdo->close_display_when_freed)
    XCloseDisplay(xdo->xdpy);

  free(xdo);
}

int xdo_wait_for_window_map_state(const xdo_t *xdo, Window wid, int map_state) {
  int tries = MAX_TRIES;
  XWindowAttributes attr;
  attr.map_state = IsUnmapped;
  while (tries > 0 && attr.map_state != map_state) {
    XGetWindowAttributes(xdo->xdpy, wid, &attr);
    usleep(30000); /* TODO(sissel): Use exponential backoff up to 1 second */
    tries--;
  }
  return 0;
}

int xdo_map_window(const xdo_t *xdo, Window wid) {
  int ret = 0;
  ret = XMapWindow(xdo->xdpy, wid);
  XFlush(xdo->xdpy);
  return _is_success("XMapWindow", ret == 0, xdo);
}

int xdo_unmap_window(const xdo_t *xdo, Window wid) {
  int ret = 0;
  ret = XUnmapWindow(xdo->xdpy, wid);
  XFlush(xdo->xdpy);
  return _is_success("XUnmapWindow", ret == 0, xdo);
}

int xdo_reparent_window(const xdo_t *xdo, Window wid_source, Window wid_target) {
  int ret = 0;
  ret = XReparentWindow(xdo->xdpy, wid_source, wid_target, 0, 0);
  XFlush(xdo->xdpy);
  return _is_success("XReparentWindow", ret == 0, xdo);
}

int xdo_get_window_location(const xdo_t *xdo, Window wid,
                            int *x_ret, int *y_ret, Screen **screen_ret) {
  int ret;
  XWindowAttributes attr;
  ret = XGetWindowAttributes(xdo->xdpy, wid, &attr);
  if (ret != 0) {
    int x, y;
    Window unused_child;

    /* The coordinates in attr are relative to the parent window.  If
     * the parent window is the root window, then the coordinates are
     * correct.  If the parent window isn't the root window --- which
     * is likely --- then we translate them. */
    Window parent;
    Window root;
    Window* children;
    unsigned int nchildren;
    XQueryTree(xdo->xdpy, wid, &root, &parent, &children, &nchildren);
    if (children != NULL) {
      XFree(children);
    }
    if (parent == attr.root) {
      x = attr.x;
      y = attr.y;
    } else {
      XTranslateCoordinates(xdo->xdpy, wid, attr.root,
                            0, 0, &x, &y, &unused_child);
    }

    if (x_ret != NULL) {
      *x_ret = x;
    }

    if (y_ret != NULL) {
      *y_ret = y;
    }

    if (screen_ret != NULL) {
      *screen_ret = attr.screen;
    }
  }
  return _is_success("XGetWindowAttributes", ret == 0, xdo);
}

int xdo_get_window_size(const xdo_t *xdo, Window wid, unsigned int *width_ret,
                        unsigned int *height_ret) {
  int ret;
  XWindowAttributes attr;
  ret = XGetWindowAttributes(xdo->xdpy, wid, &attr);
  if (ret != 0) {
    if (width_ret != NULL) {
      *width_ret = attr.width;
    }

    if (height_ret != NULL) {
      *height_ret = attr.height;
    }
  }
  return _is_success("XGetWindowAttributes", ret == 0, xdo);
}

int xdo_move_window(const xdo_t *xdo, Window wid, int x, int y) {
  XWindowChanges wc;
  int ret = 0;
  wc.x = x;
  wc.y = y;

  ret = XConfigureWindow(xdo->xdpy, wid, CWX | CWY, &wc);
  return _is_success("XConfigureWindow", ret == 0, xdo);
}

int xdo_translate_window_with_sizehint(const xdo_t *xdo, Window window,
                                       unsigned int width, unsigned int height, 
                                       unsigned int *width_ret, unsigned int *height_ret) {
  XSizeHints hints;
  long supplied_return;
  XGetWMNormalHints(xdo->xdpy, window, &hints, &supplied_return);
  if (supplied_return & PResizeInc) {
    width *= hints.width_inc;
    height *= hints.height_inc;
  } else {
    fprintf(stderr, "No size hints found for window %ld\n", window);
    *width_ret = width;
    *height_ret = width;
  }

  if (supplied_return & PBaseSize) {
    width += hints.base_width;
    height += hints.base_height;
  }

  if (width_ret != NULL) {
    *width_ret = width;
  }

  if (height_ret != NULL) {
    *height_ret = height;
  }

  return XDO_SUCCESS;
}

int xdo_set_window_size(const xdo_t *xdo, Window window, int width, int height, int flags) {
  XWindowChanges wc;
  int ret = 0;
  int cw_flags = 0;

  if (flags & SIZE_USEHINTS) {
    flags |= SIZE_USEHINTS_X | SIZE_USEHINTS_Y;
  }

  wc.width = width;
  wc.height = height;

  if (flags & SIZE_USEHINTS_X) {
    xdo_translate_window_with_sizehint(xdo, window, width, height, (unsigned int*)&wc.width,
                                       NULL);
  }

  if (flags & SIZE_USEHINTS_Y) {
    xdo_translate_window_with_sizehint(xdo, window, width, height, NULL,
                                       (unsigned int*)&wc.height);
  }

  if (width > 0) {
    cw_flags |= CWWidth;
  }

  if (height > 0) {
    cw_flags |= CWHeight;
  }

  ret = XConfigureWindow(xdo->xdpy, window, cw_flags, &wc);
  XFlush(xdo->xdpy);
  return _is_success("XConfigureWindow", ret == 0, xdo);
}

int xdo_set_window_override_redirect(const xdo_t *xdo, Window wid,
                                     int override_redirect) {
  int ret;
  XSetWindowAttributes wattr;
  long mask = CWOverrideRedirect;
  wattr.override_redirect = override_redirect;
  ret = XChangeWindowAttributes(xdo->xdpy, wid, mask, &wattr);

  return _is_success("XChangeWindowAttributes", ret == 0, xdo);
}

int xdo_set_window_class (const xdo_t *xdo, Window wid, const char *name,
                         const char *_class) {
  int ret = 0;
  XClassHint *hint = XAllocClassHint();
  XGetClassHint(xdo->xdpy, wid, hint);
  if (name != NULL)
    hint->res_name = (char*)name;

  if(_class != NULL)
    hint->res_class = (char*)_class;

  ret = XSetClassHint(xdo->xdpy, wid, hint);
  XFree(hint);
  return _is_success("XSetClassHint", ret == 0, xdo);
}

int xdo_set_window_urgency (const xdo_t *xdo, Window wid, int urgency) {
  int ret = 0;
  XWMHints *hint = XGetWMHints(xdo->xdpy, wid);
  if (hint == NULL)
    hint = XAllocWMHints();

  if (urgency)
    hint->flags = hint->flags | XUrgencyHint;
  else
    hint->flags = hint->flags & ~XUrgencyHint;

  ret = XSetWMHints(xdo->xdpy, wid, hint);
  XFree(hint);
  return _is_success("XSetWMHint", ret == 0, xdo);
}

int xdo_set_window_property(const xdo_t *xdo, Window wid, const char *property, const char *value) {
  
  char netwm_property[256] = "_NET_";
  int ret = 0;
  strcat(netwm_property, property);

  // Change the property
  ret = XChangeProperty(xdo->xdpy, wid, 
                        XInternAtom(xdo->xdpy, property, False), 
                        XInternAtom(xdo->xdpy, "STRING", False), 8, 
                        PropModeReplace, (unsigned char*)value, strlen(value));
  if (ret == 0) {
    return _is_success("XChangeProperty", ret == 0, xdo);
  }

  // Change _NET_<property> just in case for simpler NETWM compliance?
  ret = XChangeProperty(xdo->xdpy, wid, 
                        XInternAtom(xdo->xdpy, netwm_property, False), 
                        XInternAtom(xdo->xdpy, "STRING", False), 8, 
                        PropModeReplace, (unsigned char*)value, strlen(value));
  return _is_success("XChangeProperty", ret == 0, xdo);
}

int xdo_focus_window(const xdo_t *xdo, Window wid) {
  int ret = 0;
  ret = XSetInputFocus(xdo->xdpy, wid, RevertToParent, CurrentTime);
  XFlush(xdo->xdpy);
  return _is_success("XSetInputFocus", ret == 0, xdo);
}

int xdo_wait_for_window_size(const xdo_t *xdo, Window window,
                             unsigned int width, unsigned int height,
                             int flags, int to_or_from) {
  unsigned int cur_width, cur_height;
  /*unsigned int alt_width, alt_height;*/

  //printf("Want: %udx%ud\n", width, height);
  if (flags & SIZE_USEHINTS) {
    xdo_translate_window_with_sizehint(xdo, window, width, height,
                                       &width, &height);
  } else {
    unsigned int hint_width, hint_height;
    /* TODO(sissel): fix compiler warning here, but it will require
     * an ABI breakage by changing types... */
    xdo_translate_window_with_sizehint(xdo, window, 1, 1,
                                       &hint_width, &hint_height);
    //printf("Hint: %dx%d\n", hint_width, hint_height);
    /* Find the nearest multiple (rounded down) of the hint height. */
    /*alt_width = (width - (width % hint_width));*/
    /*alt_height = (height - (height % hint_height));*/
    //printf("Alt: %udx%ud\n", alt_width, alt_height);
  }

  int tries = MAX_TRIES;
  xdo_get_window_size(xdo, window, &cur_width,
                      &cur_height);
  //printf("Want: %udx%ud\n", width, height);
  //printf("Alt: %udx%ud\n", alt_width, alt_height);
  while (tries > 0 && (to_or_from == SIZE_TO
         ? (cur_width != width && cur_height != height)
         : (cur_width == width && cur_height == height))) {
    xdo_get_window_size(xdo, window, (unsigned int *)&cur_width,
                        (unsigned int *)&cur_height);
    usleep(30000);
    tries--;
  }

  return 0;
}

int xdo_wait_for_window_active(const xdo_t *xdo, Window window, int active) {
  Window activewin = 0;
  int ret = 0;
  int tries = MAX_TRIES;

  /* If active is true, wait until activewin is our window
   * otherwise, wait until activewin is not our window */
  while (tries > 0 && 
         (active ? activewin != window : activewin == window)) {
    ret = xdo_get_active_window(xdo, &activewin);
    if (ret == XDO_ERROR) {
      return ret;
    }
    usleep(30000);
    tries--;
  }

  return 0;
}

int xdo_activate_window(const xdo_t *xdo, Window wid) {
  int ret = 0;
  long desktop = 0;
  XEvent xev;
  XWindowAttributes wattr;

  if (_xdo_ewmh_is_supported(xdo, "_NET_ACTIVE_WINDOW") == False) {
    fprintf(stderr,
            "Your windowmanager claims not to support _NET_ACTIVE_WINDOW, "
            "so the attempt to activate the window was aborted.\n");
    return XDO_ERROR;
  }

  /* If this window is on another desktop, let's go to that desktop first */

  if (_xdo_ewmh_is_supported(xdo, "_NET_WM_DESKTOP") == True
      && _xdo_ewmh_is_supported(xdo, "_NET_CURRENT_DESKTOP") == True) {
    xdo_get_desktop_for_window(xdo, wid, &desktop);
    xdo_set_current_desktop(xdo, desktop);
  }

  memset(&xev, 0, sizeof(xev));
  xev.type = ClientMessage;
  xev.xclient.display = xdo->xdpy;
  xev.xclient.window = wid;
  xev.xclient.message_type = XInternAtom(xdo->xdpy, "_NET_ACTIVE_WINDOW", False);
  xev.xclient.format = 32;
  xev.xclient.data.l[0] = 2L; /* 2 == Message from a window pager */
  xev.xclient.data.l[1] = CurrentTime;

  XGetWindowAttributes(xdo->xdpy, wid, &wattr);
  ret = XSendEvent(xdo->xdpy, wattr.screen->root, False,
                   SubstructureNotifyMask | SubstructureRedirectMask,
                   &xev);

  /* XXX: XSendEvent returns 0 on conversion failure, nonzero otherwise.
   * Manpage says it will only generate BadWindow or BadValue errors */
  return _is_success("XSendEvent[EWMH:_NET_ACTIVE_WINDOW]", ret == 0, xdo);
}

int xdo_set_number_of_desktops(const xdo_t *xdo, long ndesktops) {
  /* XXX: This should support passing a screen number */
  XEvent xev;
  Window root;
  int ret = 0;

  if (_xdo_ewmh_is_supported(xdo, "_NET_NUMBER_OF_DESKTOPS") == False) {
    fprintf(stderr,
            "Your windowmanager claims not to support _NET_NUMBER_OF_DESKTOPS, "
            "so the attempt to change the number of desktops was aborted.\n");
    return XDO_ERROR;
  }

  root = RootWindow(xdo->xdpy, 0);

  memset(&xev, 0, sizeof(xev));
  xev.type = ClientMessage;
  xev.xclient.display = xdo->xdpy;
  xev.xclient.window = root;
  xev.xclient.message_type = XInternAtom(xdo->xdpy, "_NET_NUMBER_OF_DESKTOPS", 
                                         False);
  xev.xclient.format = 32;
  xev.xclient.data.l[0] = ndesktops;

  ret = XSendEvent(xdo->xdpy, root, False,
                   SubstructureNotifyMask | SubstructureRedirectMask,
                   &xev);

  return _is_success("XSendEvent[EWMH:_NET_NUMBER_OF_DESKTOPS]", ret == 0, xdo);
}

int xdo_get_number_of_desktops(const xdo_t *xdo, long *ndesktops) {
  Atom type;
  int size;
  long nitems;
  unsigned char *data;
  Window root;
  Atom request;

  if (_xdo_ewmh_is_supported(xdo, "_NET_NUMBER_OF_DESKTOPS") == False) {
    fprintf(stderr,
            "Your windowmanager claims not to support _NET_NUMBER_OF_DESKTOPS, "
            "so the attempt to query the number of desktops was aborted.\n");
    return XDO_ERROR;
  }

  request = XInternAtom(xdo->xdpy, "_NET_NUMBER_OF_DESKTOPS", False);
  root = XDefaultRootWindow(xdo->xdpy);

  data = xdo_get_window_property_by_atom(xdo, root, request, &nitems, &type, &size);

  if (nitems > 0) {
    *ndesktops = *((long*)data);
  } else {
    *ndesktops = 0;
  }
  free(data);

  return _is_success("XGetWindowProperty[_NET_NUMBER_OF_DESKTOPS]",
                     *ndesktops == 0, xdo);
}

int xdo_set_current_desktop(const xdo_t *xdo, long desktop) {
  /* XXX: This should support passing a screen number */
  XEvent xev;
  Window root;
  int ret = 0;

  root = RootWindow(xdo->xdpy, 0);

  if (_xdo_ewmh_is_supported(xdo, "_NET_CURRENT_DESKTOP") == False) {
    fprintf(stderr,
            "Your windowmanager claims not to support _NET_CURRENT_DESKTOP, "
            "so the attempt to change desktops was aborted.\n");
    return XDO_ERROR;
  }

  memset(&xev, 0, sizeof(xev));
  xev.type = ClientMessage;
  xev.xclient.display = xdo->xdpy;
  xev.xclient.window = root;
  xev.xclient.message_type = XInternAtom(xdo->xdpy, "_NET_CURRENT_DESKTOP", 
                                         False);
  xev.xclient.format = 32;
  xev.xclient.data.l[0] = desktop;
  xev.xclient.data.l[1] = CurrentTime;

  ret = XSendEvent(xdo->xdpy, root, False,
                   SubstructureNotifyMask | SubstructureRedirectMask,
                   &xev);

  return _is_success("XSendEvent[EWMH:_NET_CURRENT_DESKTOP]", ret == 0, xdo);
}

int xdo_get_current_desktop(const xdo_t *xdo, long *desktop) {
  Atom type;
  int size;
  long nitems;
  unsigned char *data;
  Window root;

  Atom request;

  if (_xdo_ewmh_is_supported(xdo, "_NET_CURRENT_DESKTOP") == False) {
    fprintf(stderr,
            "Your windowmanager claims not to support _NET_CURRENT_DESKTOP, "
            "so the query for the current desktop was aborted.\n");
    return XDO_ERROR;
  }

  request = XInternAtom(xdo->xdpy, "_NET_CURRENT_DESKTOP", False);
  root = XDefaultRootWindow(xdo->xdpy);

  data = xdo_get_window_property_by_atom(xdo, root, request, &nitems, &type, &size);

  if (nitems > 0) {
    *desktop = *((long*)data);
  } else {
    *desktop = -1;
  }
  free(data);

  return _is_success("XGetWindowProperty[_NET_CURRENT_DESKTOP]",
                     *desktop == -1, xdo);
}

int xdo_set_desktop_for_window(const xdo_t *xdo, Window wid, long desktop) {
  XEvent xev;
  int ret = 0;
  XWindowAttributes wattr;
  XGetWindowAttributes(xdo->xdpy, wid, &wattr);

  if (_xdo_ewmh_is_supported(xdo, "_NET_WM_DESKTOP") == False) {
    fprintf(stderr,
            "Your windowmanager claims not to support _NET_WM_DESKTOP, "
            "so the attempt to change a window's desktop location was "
            "aborted.\n");
    return XDO_ERROR;
  }

  memset(&xev, 0, sizeof(xev));
  xev.type = ClientMessage;
  xev.xclient.display = xdo->xdpy;
  xev.xclient.window = wid;
  xev.xclient.message_type = XInternAtom(xdo->xdpy, "_NET_WM_DESKTOP", 
                                         False);
  xev.xclient.format = 32;
  xev.xclient.data.l[0] = desktop;
  xev.xclient.data.l[1] = 2; /* indicate we are messaging from a pager */

  ret = XSendEvent(xdo->xdpy, wattr.screen->root, False,
                   SubstructureNotifyMask | SubstructureRedirectMask,
                   &xev);

  return _is_success("XSendEvent[EWMH:_NET_WM_DESKTOP]", ret == 0, xdo);
}

int xdo_get_desktop_for_window(const xdo_t *xdo, Window wid, long *desktop) {
  Atom type;
  int size;
  long nitems;
  unsigned char *data;
  Atom request;

  if (_xdo_ewmh_is_supported(xdo, "_NET_WM_DESKTOP") == False) {
    fprintf(stderr,
            "Your windowmanager claims not to support _NET_WM_DESKTOP, "
            "so the attempt to query a window's desktop location was "
            "aborted.\n");
    return XDO_ERROR;
  }

  request = XInternAtom(xdo->xdpy, "_NET_WM_DESKTOP", False);

  data = xdo_get_window_property_by_atom(xdo, wid, request, &nitems, &type, &size);

  if (nitems > 0) {
    *desktop = *((long*)data);
  } else {
    *desktop = -1;
  }
  free(data);

  return _is_success("XGetWindowProperty[_NET_WM_DESKTOP]",
                     *desktop == -1, xdo);
}

int xdo_get_active_window(const xdo_t *xdo, Window *window_ret) {
  Atom type;
  int size;
  long nitems;
  unsigned char *data;
  Atom request;
  Window root;

  if (_xdo_ewmh_is_supported(xdo, "_NET_ACTIVE_WINDOW") == False) {
    fprintf(stderr,
            "Your windowmanager claims not to support _NET_ACTIVE_WINDOW, "
            "so the attempt to query the active window aborted.\n");
    return XDO_ERROR;
  }

  request = XInternAtom(xdo->xdpy, "_NET_ACTIVE_WINDOW", False);
  root = XDefaultRootWindow(xdo->xdpy);
  data = xdo_get_window_property_by_atom(xdo, root, request, &nitems, &type, &size);

  if (nitems > 0) {
    *window_ret = *((Window*)data);
  } else {
    *window_ret = 0;
  }
  free(data);

  return _is_success("XGetWindowProperty[_NET_ACTIVE_WINDOW]",
                     *window_ret == 0, xdo);
}

int xdo_select_window_with_click(const xdo_t *xdo, Window *window_ret) {
  int screen_num;
  Screen *screen;
  xdo_get_mouse_location(xdo, NULL, NULL, &screen_num);

  screen = ScreenOfDisplay(xdo->xdpy, screen_num);

  /* Grab sync mode so we can ensure nothing changes while we figure
   * out what the client window is.
   * Also, everyone else who does 'select window' does it this way.
   */
  Cursor cursor = XCreateFontCursor(xdo->xdpy, XC_target);
  int grab_ret = 0;
  grab_ret = XGrabPointer(xdo->xdpy, screen->root, False, ButtonReleaseMask,
               GrabModeSync, GrabModeAsync, screen->root, cursor, CurrentTime);
  if (grab_ret == AlreadyGrabbed) {
    fprintf(stderr, "Attempt to grab the mouse failed. Something already has"
            " the mouse grabbed. This can happen if you are dragging something"
            " or if there is a popup currently shown\n");
    return XDO_ERROR;
  }

  XEvent e;
  XAllowEvents(xdo->xdpy, SyncPointer, CurrentTime);
  XWindowEvent(xdo->xdpy, screen->root, ButtonReleaseMask, &e);
  XUngrabPointer(xdo->xdpy, CurrentTime);
  XFreeCursor(xdo->xdpy, cursor);

  if (e.xbutton.button != 1) {
    fprintf(stderr, "window selection aborted with button %d\n", e.xbutton.button);
    return XDO_ERROR;
  }

  /* If there is no subwindow, then we clicked on the root window */
  if (e.xbutton.subwindow == 0) {
    *window_ret = e.xbutton.root;
  } else {
     /* Random testing showed that 'root' always is the same as 'window'
      * while 'subwindow' is the actual window we clicked on. Confusing... */
     *window_ret = e.xbutton.subwindow;
    _xdo_debug(xdo, "Click on window %lu foo", *window_ret);
    xdo_find_window_client(xdo, *window_ret, window_ret, XDO_FIND_CHILDREN);
  }
  return XDO_SUCCESS;
}

/* XRaiseWindow is ignored in ion3 and Gnome2. Is it even useful? */
int xdo_raise_window(const xdo_t *xdo, Window wid) {
  int ret = 0;
  ret = XRaiseWindow(xdo->xdpy, wid);
  XFlush(xdo->xdpy);
  return _is_success("XRaiseWindow", ret == 0, xdo);
}

int xdo_move_mouse(const xdo_t *xdo, int x, int y, int screen)  {
  int ret = 0;

  /* There is a bug (feature?) in XTestFakeMotionEvent that causes
   * the screen number in the request to be ignored. The internets
   * seem to recommend XWarpPointer instead, ie;
   * https://bugzilla.redhat.com/show_bug.cgi?id=518803
   */
  Window screen_root = RootWindow(xdo->xdpy, screen);
  ret = XWarpPointer(xdo->xdpy, None, screen_root, 0, 0, 0, 0, x, y);
  XFlush(xdo->xdpy);
  return _is_success("XWarpPointer", ret == 0, xdo);
}

int xdo_move_mouse_relative_to_window(const xdo_t *xdo, Window window, int x, int y) {
  XWindowAttributes attr;
  Window unused_child;
  int root_x, root_y;

  XGetWindowAttributes(xdo->xdpy, window, &attr);
  XTranslateCoordinates(xdo->xdpy, window, attr.root,
                        x, y, &root_x, &root_y, &unused_child);
  return xdo_move_mouse(xdo, root_x, root_y, XScreenNumberOfScreen(attr.screen));
}

int xdo_move_mouse_relative(const xdo_t *xdo, int x, int y)  {
  int ret = 0;
  ret = XTestFakeRelativeMotionEvent(xdo->xdpy, x, y, CurrentTime);
  XFlush(xdo->xdpy);
  return _is_success("XTestFakeRelativeMotionEvent", ret == 0, xdo);
}

int _xdo_mousebutton(const xdo_t *xdo, Window window, int button, int is_press) {
  int ret = 0;

  if (window == CURRENTWINDOW) {
    ret = XTestFakeButtonEvent(xdo->xdpy, button, is_press, CurrentTime);
    XFlush(xdo->xdpy);
    return _is_success("XTestFakeButtonEvent(down)", ret == 0, xdo);
  } else {
    /* Send to specific window */
    int screen = 0;
    XButtonEvent xbpe;
    charcodemap_t *active_mod;
    int active_mod_n;

    xdo_get_mouse_location(xdo, &xbpe.x_root, &xbpe.y_root, &screen);
    xdo_get_active_modifiers(xdo, &active_mod, &active_mod_n);

    xbpe.window = window;
    xbpe.button = button;
    xbpe.display = xdo->xdpy;
    xbpe.root = RootWindow(xdo->xdpy, screen);
    xbpe.same_screen = True; /* Should we detect if window is on the same
                                 screen as cursor? */
    xbpe.state = xdo_get_input_state(xdo);

    xbpe.subwindow = None;
    xbpe.time = CurrentTime;
    xbpe.type = (is_press ? ButtonPress : ButtonRelease);

    /* Get the coordinates of the cursor relative to xbpe.window and also find what
     * subwindow it might be on */
    XTranslateCoordinates(xdo->xdpy, xbpe.root, xbpe.window, 
                          xbpe.x_root, xbpe.y_root, &xbpe.x, &xbpe.y, &xbpe.subwindow);

    /* Normal behavior of 'mouse up' is that the modifier mask includes
     * 'ButtonNMotionMask' where N is the button being released. This works the
     * same way with keys, too. */
    if (!is_press) { /* is mouse up */
      switch(button) {
        case 1: xbpe.state |= Button1MotionMask; break;
        case 2: xbpe.state |= Button2MotionMask; break;
        case 3: xbpe.state |= Button3MotionMask; break;
        case 4: xbpe.state |= Button4MotionMask; break;
        case 5: xbpe.state |= Button5MotionMask; break;
      }
    }
    ret = XSendEvent(xdo->xdpy, window, True, ButtonPressMask, (XEvent *)&xbpe);
    XFlush(xdo->xdpy);
    free(active_mod);
    return _is_success("XSendEvent(mousedown)", ret == 0, xdo);
  }
}

int xdo_mouse_up(const xdo_t *xdo, Window window, int button) {
  return _xdo_mousebutton(xdo, window, button, False);
}

int xdo_mouse_down(const xdo_t *xdo, Window window, int button) {
  return _xdo_mousebutton(xdo, window, button, True);
}

int xdo_get_mouse_location(const xdo_t *xdo, int *x_ret, int *y_ret,
                           int *screen_num_ret) {
  return xdo_get_mouse_location2(xdo, x_ret, y_ret, screen_num_ret, NULL);
}

int xdo_get_window_at_mouse(const xdo_t *xdo, Window *window_ret) {
  return xdo_get_mouse_location2(xdo, NULL, NULL, NULL, window_ret);
}

int xdo_get_mouse_location2(const xdo_t *xdo, int *x_ret, int *y_ret,
                            int *screen_num_ret, Window *window_ret) {
  int ret = False;
  int x = 0, y = 0, screen_num = 0;
  int i = 0;
  Window window = 0;
  Window root = 0;
  int dummy_int = 0;
  unsigned int dummy_uint = 0;
  int screencount = ScreenCount(xdo->xdpy);

  for (i = 0; i < screencount; i++) {
    Screen *screen = ScreenOfDisplay(xdo->xdpy, i);
    ret = XQueryPointer(xdo->xdpy, RootWindowOfScreen(screen),
                        &root, &window,
                        &x, &y, &dummy_int, &dummy_int, &dummy_uint);
    if (ret == True) {
      screen_num = i;
      break;
    }
  }

  if (window_ret != NULL) {
    /* Find the client window if we are not root. */
    if (window != root && window != 0) {
      int findret;
      Window client = 0;

      /* Search up the stack for a client window for this window */
      findret = xdo_find_window_client(xdo, window, &client, XDO_FIND_PARENTS);
      if (findret == XDO_ERROR) {
        /* If no client found, search down the stack */
        findret = xdo_find_window_client(xdo, window, &client, XDO_FIND_CHILDREN);
      }
      //fprintf(stderr, "%ld, %ld, %ld, %d\n", window, root, client, findret);
      if (findret == XDO_SUCCESS) {
        window = client;
      }
    } else {
      window = root;
    }
  }
  //printf("mouseloc root: %ld\n", root);
  //printf("mouseloc window: %ld\n", window);

  if (ret == True) {
    if (x_ret != NULL) *x_ret = x;
    if (y_ret != NULL) *y_ret = y;
    if (screen_num_ret != NULL) *screen_num_ret = screen_num;
    if (window_ret != NULL) *window_ret = window;
  }

  return _is_success("XQueryPointer", ret == False, xdo);
}

int xdo_click_window(const xdo_t *xdo, Window window, int button) {
  int ret = 0;
  ret = xdo_mouse_down(xdo, window, button);
  if (ret != XDO_SUCCESS) {
    fprintf(stderr, "xdo_mouse_down failed, aborting click.\n");
    return ret;
  }
  usleep(DEFAULT_DELAY);
  ret = xdo_mouse_up(xdo, window, button);
  return ret;
}

int xdo_click_window_multiple(const xdo_t *xdo, Window window, int button,
                       int repeat, useconds_t delay) {
  int ret = 0;
  while (repeat > 0) {
    ret = xdo_click_window(xdo, window, button);
    if (ret != XDO_SUCCESS) {
      fprintf(stderr, "click failed with %d repeats remaining\n", repeat);
      return ret;
    }
    repeat--;

    /* Sleeping even after the last click is important, so that a call to xdo_set_active_modifiers()
     * right after won't think that the button is still pressed. */
    usleep(delay);
  } /* while (repeat > 0) */
  return ret;
} /* int xdo_click_window_multiple */

/* XXX: Return proper code if errors found */
int xdo_enter_text_window(const xdo_t *xdo, Window window, const char *string, useconds_t delay) {

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
    _xdo_charcodemap_from_char(xdo, &key);
    if (key.code == 0 && key.symbol == NoSymbol) {
      fprintf(stderr, "I don't know which key produces '%lc', skipping.\n",
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
    xdo_send_keysequence_window_list_do(xdo, window, &key, 1, True, NULL, delay / 2);
    key.needs_binding = 0;
    xdo_send_keysequence_window_list_do(xdo, window, &key, 1, False, NULL, delay / 2);

    /* XXX: Flush here or at the end? or never? */
    //XFlush(xdo->xdpy);
  } /* walk string generating a keysequence */

  //free(keys);
  return XDO_SUCCESS;
}

int _xdo_send_keysequence_window_do(const xdo_t *xdo, Window window, const char *keyseq,
                        int pressed, int *modifier, useconds_t delay) {
  int ret = 0;
  charcodemap_t *keys = NULL;
  int nkeys = 0;

  if (_xdo_send_keysequence_window_to_keycode_list(xdo, keyseq, &keys, &nkeys) == False) {
    fprintf(stderr, "Failure converting key sequence '%s' to keycodes\n", keyseq);
    return 1;
  }

  ret = xdo_send_keysequence_window_list_do(xdo, window, keys, nkeys, pressed, modifier, delay);
  free(keys);

  return ret;
}

int xdo_send_keysequence_window_list_do(const xdo_t *xdo, Window window, charcodemap_t *keys, 
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
      _xdo_debug(xdo, "Mapping sym %lu to %d", keys[i].symbol, scratch_keycode);
      XChangeKeyboardMapping(xdo->xdpy, scratch_keycode, 1, keysym_list, 1);
      XSync(xdo->xdpy, False);
      /* override the code in our current key to use the scratch_keycode */
      keys[i].code = scratch_keycode;
      keymapchanged = 1;
    }

    //fprintf(stderr, "keyseqlist_do: Sending %lc %s (%d, mods %x)\n",
            //keys[i].key, (pressed ? "down" : "up"), keys[i].code, *modifier);
    _xdo_send_key(xdo, window, &(keys[i]), *modifier, pressed, delay);

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
    _xdo_debug(xdo, "Reverting scratch keycode (sym %lu to %d)",
              keys[i].symbol, scratch_keycode);
    XChangeKeyboardMapping(xdo->xdpy, scratch_keycode, 1, keysym_list, 1);
  }

  /* Necessary? */
  XFlush(xdo->xdpy);
  return XDO_SUCCESS;
}

  
int xdo_send_keysequence_window_down(const xdo_t *xdo, Window window, const char *keyseq,
                         useconds_t delay) {
  return _xdo_send_keysequence_window_do(xdo, window, keyseq, True, NULL, delay);
}

int xdo_send_keysequence_window_up(const xdo_t *xdo, Window window, const char *keyseq,
                       useconds_t delay) {
  return _xdo_send_keysequence_window_do(xdo, window, keyseq, False, NULL, delay);
}

int xdo_send_keysequence_window(const xdo_t *xdo, Window window, const char *keyseq,
                    useconds_t delay) {
  int ret = 0;
  int modifier = 0;
  ret += _xdo_send_keysequence_window_do(xdo, window, keyseq, True, &modifier, delay / 2);
  ret += _xdo_send_keysequence_window_do(xdo, window, keyseq, False, &modifier, delay / 2);
  return ret;
}

/* Add by Lee Pumphret 2007-07-28
 * Modified slightly by Jordan Sissel */
int xdo_get_focused_window(const xdo_t *xdo, Window *window_ret) {
  int ret = 0;
  int unused_revert_ret;

  ret = XGetInputFocus(xdo->xdpy, window_ret, &unused_revert_ret);

  /* Xvfb with no window manager and given otherwise no input, with 
   * a single client, will return the current focused window as '1'
   * I think this is a bug, so let's alert the user. */
  if (*window_ret == 1) {
    fprintf(stderr, 
            "XGetInputFocus returned the focused window of %ld. "
            "This is likely a bug in the X server.\n", *window_ret);
  }
  return _is_success("XGetInputFocus", ret == 0, xdo);
}

int xdo_wait_for_window_focus(const xdo_t *xdo, Window window, int want_focus) {
  Window focuswin = 0;
  int ret;
  int tries = MAX_TRIES;
  ret = xdo_get_focused_window(xdo, &focuswin);
  if (ret != 0) {
    return ret;
  }

  while (tries > 0 && 
         (want_focus ? focuswin != window : focuswin == window)) {
    usleep(30000); /* TODO(sissel): Use exponential backoff up to 1 second */
    ret = xdo_get_focused_window(xdo, &focuswin);
    if (ret != 0) {
      return ret;
    }
    tries--;
  }
  return 0;
}

/* Like xdo_get_focused_window, but return the first ancestor-or-self window
 * having a property of WM_CLASS. This allows you to get the "real" or
 * top-level-ish window having focus rather than something you may
 * not expect to be the window having focused. */
int xdo_get_focused_window_sane(const xdo_t *xdo, Window *window_ret) {
  xdo_get_focused_window(xdo, window_ret);
  xdo_find_window_client(xdo, *window_ret, window_ret, XDO_FIND_PARENTS);
  return _is_success("xdo_get_focused_window_sane", *window_ret == 0, xdo);
}

int xdo_find_window_client(const xdo_t *xdo, Window window, Window *window_ret,
                           int direction) {
  /* for XQueryTree */
  Window dummy, parent, *children = NULL;
  unsigned int nchildren;
  Atom atom_wmstate = XInternAtom(xdo->xdpy, "WM_STATE", False);

  int done = False;
  while (!done) {
    if (window == 0) {
      return XDO_ERROR;
    }

    long items;
    _xdo_debug(xdo, "get_window_property on %lu", window);
    xdo_get_window_property_by_atom(xdo, window, atom_wmstate, &items, NULL, NULL);

    if (items == 0) {
      /* This window doesn't have WM_STATE property, keep searching. */
      _xdo_debug(xdo, "window %lu has no WM_STATE property, digging more.", window);
      XQueryTree(xdo->xdpy, window, &dummy, &parent, &children, &nchildren);

      if (direction == XDO_FIND_PARENTS) {
        _xdo_debug(xdo, "searching parents");
        /* Don't care about the children, but we still need to free them */
        if (children != NULL)
          XFree(children);
        window = parent;
      } else if (direction == XDO_FIND_CHILDREN) {
        _xdo_debug(xdo, "searching %d children", nchildren);
        unsigned int i = 0;
        int ret;
        done = True; /* recursion should end us */
        for (i = 0; i < nchildren; i++) {
          ret = xdo_find_window_client(xdo, children[i], &window, direction);
          //fprintf(stderr, "findclient: %ld\n", window);
          if (ret == XDO_SUCCESS) {
            *window_ret = window;
            break;
          }
        }
        if (nchildren == 0) {
          return XDO_ERROR;
        }
        if (children != NULL)
          XFree(children);
      } else {
        fprintf(stderr, "Invalid find_client direction (%d)\n", direction);
        *window_ret = 0;
        if (children != NULL)
          XFree(children);
        return XDO_ERROR;
      }
    } else {
      *window_ret = window;
      done = True;
    }
  }
  return XDO_SUCCESS;
}

/* Helper functions */
static KeySym _xdo_keysym_from_char(const xdo_t *xdo, wchar_t key) {
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

static void _xdo_charcodemap_from_char(const xdo_t *xdo, charcodemap_t *key) {
  KeySym keysym = _xdo_keysym_from_char(xdo, key->key);
  _xdo_charcodemap_from_keysym(xdo, key, keysym);

  /* If the character is an uppercase character within the Basic Latin or Latin-1 code block,
   * then sending the capital character keycode will not work.
   * We have to also send the shift modifier.
   * There are only three ranges of capital letters to worry about */
  if ((key->key >= 0x41 && key->key <= 0x5A) || (key->key >= 0xC0 && key->key <= 0xD6) || (key->key >= 0xD8 && key->key <= 0xDE)) {
    key->modmask = ShiftMask;
  }
}

static void _xdo_charcodemap_from_keysym(const xdo_t *xdo, charcodemap_t *key, KeySym keysym) {
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

static int _xdo_has_xtest(const xdo_t *xdo) {
  int dummy;
  return (XTestQueryExtension(xdo->xdpy, &dummy, &dummy, &dummy, &dummy) == True);
}

static void _xdo_populate_charcode_map(xdo_t *xdo) {
  /* assert xdo->display is valid */
  int keycodes_length = 0;
  int idx = 0;
  int keycode, group, groups, level, modmask, num_map;

  XDisplayKeycodes(xdo->xdpy, &(xdo->keycode_low), &(xdo->keycode_high));
  XModifierKeymap *modmap = XGetModifierMapping(xdo->xdpy);
  KeySym *keysyms = XGetKeyboardMapping(xdo->xdpy, xdo->keycode_low,
                                        xdo->keycode_high - xdo->keycode_low + 1,
                                        &xdo->keysyms_per_keycode);
  XFree(keysyms);

  /* Add 2 to the size because the range [low, high] is inclusive */
  /* Add 2 more for tab (\t) and newline (\n) */
  keycodes_length = ((xdo->keycode_high - xdo->keycode_low) + 1)
                     * xdo->keysyms_per_keycode;

  xdo->charcodes = calloc(keycodes_length, sizeof(charcodemap_t));
  XkbDescPtr desc = XkbGetMap(xdo->xdpy, XkbAllClientInfoMask, XkbUseCoreKbd);

  for (keycode = xdo->keycode_low; keycode <= xdo->keycode_high; keycode++) {
    groups = XkbKeyNumGroups(desc, keycode);
    for (group = 0; group < groups; group++) {
      XkbKeyTypePtr key_type = XkbKeyKeyType(desc, keycode, group);
      for (level = 0; level < key_type->num_levels; level++) {
        KeySym keysym = XkbKeycodeToKeysym(xdo->xdpy, keycode, group, level);
        modmask = 0;

        for (num_map = 0; num_map < key_type->map_count; num_map++) {
          XkbKTMapEntryRec map = key_type->map[num_map];
          if (map.active && map.level == level) {
            modmask = map.mods.mask;
            break;
          }
        }

        xdo->charcodes[idx].key = _keysym_to_char(keysym);
        xdo->charcodes[idx].code = keycode;
        xdo->charcodes[idx].group = group;
        xdo->charcodes[idx].modmask = modmask | _xdo_query_keycode_to_modifier(modmap, keycode);
        xdo->charcodes[idx].symbol = keysym;

        idx++;
      }
    }
  }
  xdo->charcodes_len = idx;
  XkbFreeClientMap(desc, 0, 1);
  XFreeModifiermap(modmap);
}

/* context-free functions */
wchar_t _keysym_to_char(KeySym keysym) {
  return (wchar_t)xkb_keysym_to_utf32(keysym);
}

int _xdo_send_keysequence_window_to_keycode_list(const xdo_t *xdo, const char *keyseq,
                                     charcodemap_t **keys, int *nkeys) {
  char *tokctx = NULL;
  const char *tok = NULL;
  char *keyseq_copy = NULL, *strptr = NULL;
  int i = 0;

  /* Array of keys to press, in order given by keyseq */
  int keys_size = 10;

  if (strcspn(keyseq, " \t\n.-[]{}\\|") != strlen(keyseq)) {
    fprintf(stderr, "Error: Invalid key sequence '%s'\n", keyseq);
    return False;
  }

  *nkeys = 0;
  *keys = calloc(keys_size, sizeof(charcodemap_t));
  keyseq_copy = strptr = strdup(keyseq);
  while ((tok = strtok_r(strptr, "+", &tokctx)) != NULL) {
    KeySym sym;
    KeyCode key;

    if (strptr != NULL)
      strptr = NULL;

    /* Check if 'tok' (string keysym) is an alias to another key */
    /* symbol_map comes from xdo.util */
    for (i = 0; symbol_map[i] != NULL; i+=2)
      if (!strcasecmp(tok, symbol_map[i]))
        tok = symbol_map[i + 1];

    sym = XStringToKeysym(tok);
    if (sym == NoSymbol) {
      /* Accept a number as a explicit keycode */
      if (isdigit(tok[0])) {
        key = (unsigned int) atoi(tok);
      } else {
        fprintf(stderr, "(symbol) No such key name '%s'. Ignoring it.\n", tok);
        continue;
      }
      (*keys)[*nkeys].code = key;
      (*keys)[*nkeys].symbol = sym;
      (*keys)[*nkeys].group = 0;
      (*keys)[*nkeys].modmask = 0;
      (*keys)[*nkeys].needs_binding = 0;
      if (key == 0) {
        //fprintf(stderr, "No such key '%s'. Ignoring it.\n", tok);
        (*keys)[*nkeys].needs_binding = 1;
      }
    } else {
      _xdo_charcodemap_from_keysym(xdo, &(*keys)[*nkeys], sym);
    }

    (*nkeys)++;
    if (*nkeys == keys_size) {
      keys_size *= 2;
      *keys = realloc(*keys, keys_size * sizeof(KeyCode));
    }
  }

  free(keyseq_copy);
  return True;
}

int _is_success(const char *funcname, int code, const xdo_t *xdo) {
  /* Nonzero is failure. */
  if (code != 0 && !xdo->quiet)
    fprintf(stderr, "%s failed (code=%d)\n", funcname, code);
  return code;
}

int xdo_get_window_property(const xdo_t *xdo, Window window, const char *property,
                            unsigned char **value, long *nitems, Atom *type, int *size) {
    *value = xdo_get_window_property_by_atom(xdo, window, XInternAtom(xdo->xdpy, property, False), nitems, type, size);
    if (*value == NULL) {
        return XDO_ERROR;
    }
    return XDO_SUCCESS;
}

/* Arbitrary window property retrieval
 * slightly modified version from xprop.c from Xorg */
unsigned char *xdo_get_window_property_by_atom(const xdo_t *xdo, Window window, Atom atom,
                                            long *nitems, Atom *type, int *size) {
  Atom actual_type;
  int actual_format;
  unsigned long _nitems;
  /*unsigned long nbytes;*/
  unsigned long bytes_after; /* unused */
  unsigned char *prop;
  int status;

  status = XGetWindowProperty(xdo->xdpy, window, atom, 0, (~0L),
                              False, AnyPropertyType, &actual_type,
                              &actual_format, &_nitems, &bytes_after,
                              &prop);
  if (status == BadWindow) {
    fprintf(stderr, "window id # 0x%lx does not exists!", window);
    return NULL;
  } if (status != Success) {
    fprintf(stderr, "XGetWindowProperty failed!");
    return NULL;
  }

  /*
   *if (actual_format == 32)
   *  nbytes = sizeof(long);
   *else if (actual_format == 16)
   *  nbytes = sizeof(short);
   *else if (actual_format == 8)
   *  nbytes = 1;
   *else if (actual_format == 0)
   *  nbytes = 0;
   */

  if (nitems != NULL) {
    *nitems = _nitems;
  }

  if (type != NULL) {
    *type = actual_type;
  }

  if (size != NULL) {
    *size = actual_format;
  }
  return prop;
}

int _xdo_ewmh_is_supported(const xdo_t *xdo, const char *feature) {
  Atom type = 0;
  long nitems = 0L;
  int size = 0;
  Atom *results = NULL;
  long i = 0;

  Window root;
  Atom request;
  Atom feature_atom;

  request = XInternAtom(xdo->xdpy, "_NET_SUPPORTED", False);
  feature_atom = XInternAtom(xdo->xdpy, feature, False);
  root = XDefaultRootWindow(xdo->xdpy);

  results = (Atom *) xdo_get_window_property_by_atom(xdo, root, request, &nitems, &type, &size);
  for (i = 0L; i < nitems; i++) {
    if (results[i] == feature_atom) {
      free(results);
      return True;
    }
  }
  free(results);

  return False;
}

void _xdo_init_xkeyevent(const xdo_t *xdo, XKeyEvent *xk) {
  xk->display = xdo->xdpy;
  xk->subwindow = None;
  xk->time = CurrentTime;
  xk->same_screen = True;

  /* Should we set these at all? */
  xk->x = xk->y = xk->x_root = xk->y_root = 1;
}

void _xdo_send_key(const xdo_t *xdo, Window window, charcodemap_t *key,
                          int modstate, int is_press, useconds_t delay) {
  /* Properly ensure the modstate is set by finding a key
   * that activates each bit in the modifier state */
  int mask = modstate | key->modmask;
  int use_xtest = 0;

  if (window == CURRENTWINDOW) {
    use_xtest = 1;
  } else {
    Window focuswin = 0;
    xdo_get_focused_window(xdo, &focuswin);
    if (focuswin == window) {
      use_xtest = 1;
    }
  }
  if (use_xtest) {
    //printf("XTEST: Sending key %d %s\n", key->code, is_press ? "down" : "up");
    XkbStateRec state;
    XkbGetState(xdo->xdpy, XkbUseCoreKbd, &state);
    int current_group = state.group;
    XkbLockGroup(xdo->xdpy, XkbUseCoreKbd, key->group);
    if (mask)
      _xdo_send_modifier(xdo, mask, is_press);
    //printf("XTEST: Sending key %d %s %x %d\n", key->code, is_press ? "down" : "up", key->modmask, key->group);
    XTestFakeKeyEvent(xdo->xdpy, key->code, is_press, CurrentTime);
    XkbLockGroup(xdo->xdpy, XkbUseCoreKbd, current_group);
    XSync(xdo->xdpy, False);
  } else {
    /* Since key events have 'state' (shift, etc) in the event, we don't
     * need to worry about key press ordering. */
    XKeyEvent xk;
    _xdo_init_xkeyevent(xdo, &xk);
    xk.window = window;
    xk.keycode = key->code;
    xk.state = mask | (key->group << 13);
    xk.type = (is_press ? KeyPress : KeyRelease);
    XSendEvent(xdo->xdpy, xk.window, True, KeyPressMask, (XEvent *)&xk);
  }

  /* Skipping the usleep if delay is 0 is much faster than calling usleep(0) */
  XFlush(xdo->xdpy);
  if (delay > 0) {
    usleep(delay);
  }
}

int _xdo_query_keycode_to_modifier(XModifierKeymap *modmap, KeyCode keycode) {
  int i = 0, j = 0;
  int max = modmap->max_keypermod;

  for (i = 0; i < 8; i++) { /* 8 modifier types, per XGetModifierMapping(3X) */
    for (j = 0; j < max && modmap->modifiermap[(i * max) + j]; j++) {
      if (keycode == modmap->modifiermap[(i * max) + j]) {
        switch (i) {
          case ShiftMapIndex: return ShiftMask; break;
          case LockMapIndex: return LockMask; break;
          case ControlMapIndex: return ControlMask; break;
          case Mod1MapIndex: return Mod1Mask; break;
          case Mod2MapIndex: return Mod2Mask; break;
          case Mod3MapIndex: return Mod3Mask; break;
          case Mod4MapIndex: return Mod4Mask; break;
          case Mod5MapIndex: return Mod5Mask; break;
        }
      } /* end if */
    } /* end loop j */
  } /* end loop i */

  /* No modifier found for this keycode, return no mask */
  return 0;
}

void _xdo_send_modifier(const xdo_t *xdo, int modmask, int is_press) {
  XModifierKeymap *modifiers = XGetModifierMapping(xdo->xdpy);
  int mod_index, mod_key, keycode;

  for (mod_index = ShiftMapIndex; mod_index <= Mod5MapIndex; mod_index++) {
    if (modmask & (1 << mod_index)) {
      for (mod_key = 0; mod_key < modifiers->max_keypermod; mod_key++) {
        keycode = modifiers->modifiermap[mod_index * modifiers->max_keypermod + mod_key];
        if (keycode) {
          XTestFakeKeyEvent(xdo->xdpy, keycode, is_press, CurrentTime);
          XSync(xdo->xdpy, False);
          break;
        }
      }
    }
  }

  XFreeModifiermap(modifiers);
}

int xdo_get_active_modifiers(const xdo_t *xdo, charcodemap_t **keys,
                                    int *nkeys) {
  /* For each keyboard device, if an active key is a modifier,
   * then add the keycode to the keycode list */

  char keymap[32]; /* keycode map: 256 bits */
  int keys_size = 10;
  int keycode = 0;
  int mod_index, mod_key;
  XModifierKeymap *modifiers = XGetModifierMapping(xdo->xdpy);
  *nkeys = 0;
  *keys = malloc(keys_size * sizeof(charcodemap_t));

  XQueryKeymap(xdo->xdpy, keymap);

  for (mod_index = ShiftMapIndex; mod_index <= Mod5MapIndex; mod_index++) {
    for (mod_key = 0; mod_key < modifiers->max_keypermod; mod_key++) {
      keycode = modifiers->modifiermap[mod_index * modifiers->max_keypermod + mod_key];
      if (keycode && keymap[(keycode / 8)] & (1 << (keycode % 8))) {
        /* This keycode is active and is a modifier, record it. */

        /* Zero the charcodemap_t entry before using it.
         * Fixes a bug reported by Hong-Leong Ong - where
         * 'xdotool key --clearmodifiers ...' sometimes failed trying
         * to clear modifiers that didn't exist since charcodemap_t's modmask was
         * uninitialized */
        memset(*keys + *nkeys, 0, sizeof(charcodemap_t));

        (*keys)[*nkeys].code = keycode;
        (*nkeys)++;

        if (*nkeys == keys_size) {
          keys_size *= 2;
          *keys = realloc(keys, keys_size * sizeof(charcodemap_t));
        }
      }
    }
  } 

  XFreeModifiermap(modifiers);

  return XDO_SUCCESS;
}

unsigned int xdo_get_input_state(const xdo_t *xdo) {
  Window root, dummy;
  int root_x, root_y, win_x, win_y;
  unsigned int mask;
  root = DefaultRootWindow(xdo->xdpy);

  XQueryPointer(xdo->xdpy, root, &dummy, &dummy,
                &root_x, &root_y, &win_x, &win_y, &mask);

  return mask;
}

const char **xdo_get_symbol_map(void) {
  return symbol_map;
}

int xdo_clear_active_modifiers(const xdo_t *xdo, Window window, charcodemap_t *active_mods, int active_mods_n) {
  int ret = 0;
  unsigned int input_state = xdo_get_input_state(xdo);
  xdo_send_keysequence_window_list_do(xdo, window, active_mods,
                          active_mods_n, False, NULL, DEFAULT_DELAY);

  if (input_state & Button1MotionMask)
    ret = xdo_mouse_up(xdo, window, 1);
  if (!ret && input_state & Button2MotionMask)
    ret = xdo_mouse_up(xdo, window, 2);
  if (!ret && input_state & Button3MotionMask)
    ret = xdo_mouse_up(xdo, window, 3);
  if (!ret && input_state & Button4MotionMask)
    ret = xdo_mouse_up(xdo, window, 4);
  if (!ret && input_state & Button5MotionMask)
    ret = xdo_mouse_up(xdo, window, 5);
  if (!ret && input_state & LockMask) {
    /* explicitly use down+up here since xdo_send_keysequence_window alone will track the modifiers
     * incurred by a key (like shift, or caps) and send them on the 'up' sequence.
     * That seems to break things with Caps_Lock only, so let's be explicit here. */
    ret = xdo_send_keysequence_window_down(xdo, window, "Caps_Lock", DEFAULT_DELAY);
    ret += xdo_send_keysequence_window_up(xdo, window, "Caps_Lock", DEFAULT_DELAY);
  }

  XSync(xdo->xdpy, False);
  return ret;
}

int xdo_set_active_modifiers(const xdo_t *xdo, Window window, charcodemap_t *active_mods, int active_mods_n) {
  int ret = 0;
  unsigned int input_state = xdo_get_input_state(xdo);
  xdo_send_keysequence_window_list_do(xdo, window, active_mods,
                          active_mods_n, True, NULL, DEFAULT_DELAY);
  if (input_state & Button1MotionMask)
    ret = xdo_mouse_down(xdo, window, 1);
  if (!ret && input_state & Button2MotionMask)
    ret = xdo_mouse_down(xdo, window, 2);
  if (!ret && input_state & Button3MotionMask)
    ret = xdo_mouse_down(xdo, window, 3);
  if (!ret && input_state & Button4MotionMask)
    ret = xdo_mouse_down(xdo, window, 4);
  if (!ret && input_state & Button5MotionMask)
    ret = xdo_mouse_down(xdo, window, 5);
  if (!ret && input_state & LockMask) {
    /* explicitly use down+up here since xdo_send_keysequence_window alone will track the modifiers
     * incurred by a key (like shift, or caps) and send them on the 'up' sequence.
     * That seems to break things with Caps_Lock only, so let's be explicit here. */
    ret = xdo_send_keysequence_window_down(xdo, window, "Caps_Lock", DEFAULT_DELAY);
    ret += xdo_send_keysequence_window_up(xdo, window, "Caps_Lock", DEFAULT_DELAY);
  }

  XSync(xdo->xdpy, False);
  return ret;
}

int xdo_get_pid_window(const xdo_t *xdo, Window window) {
  Atom type;
  int size;
  long nitems;
  unsigned char *data;
  int window_pid = 0;

  if (atom_NET_WM_PID == (Atom)-1) {
    atom_NET_WM_PID = XInternAtom(xdo->xdpy, "_NET_WM_PID", False);
  }

  data = xdo_get_window_property_by_atom(xdo, window, atom_NET_WM_PID, &nitems, &type, &size);

  if (nitems > 0) {
    /* The data itself is unsigned long, but everyone uses int as pid values */
    window_pid = (int) *((unsigned long *)data);
  }
  free(data);

  return window_pid;
}

int xdo_wait_for_mouse_move_from(const xdo_t *xdo, int origin_x, int origin_y) {
  int x, y;
  int ret = 0;
  int tries = MAX_TRIES;

  ret = xdo_get_mouse_location(xdo, &x, &y, NULL);
  while (tries > 0 && 
         (x == origin_x && y == origin_y)) {
    usleep(30000);
    ret = xdo_get_mouse_location(xdo, &x, &y, NULL);
    tries--;
  }

  return ret;
}

int xdo_wait_for_mouse_move_to(const xdo_t *xdo, int dest_x, int dest_y) {
  int x, y;
  int ret = 0;
  int tries = MAX_TRIES;

  ret = xdo_get_mouse_location(xdo, &x, &y, NULL);
  while (tries > 0 && (x != dest_x && y != dest_y)) {
    usleep(30000);
    ret = xdo_get_mouse_location(xdo, &x, &y, NULL);
    tries--;
  }

  return ret;
}

int xdo_get_desktop_viewport(const xdo_t *xdo, int *x_ret, int *y_ret) {
  if (_xdo_ewmh_is_supported(xdo, "_NET_DESKTOP_VIEWPORT") == False) {
    fprintf(stderr,
            "Your windowmanager claims not to support _NET_DESKTOP_VIEWPORT, "
            "so I cannot tell you the viewport position.\n");
    return XDO_ERROR;
  }

  Atom type;
  int size;
  long nitems;
  unsigned char *data;
  Atom request = XInternAtom(xdo->xdpy, "_NET_DESKTOP_VIEWPORT", False);
  Window root = RootWindow(xdo->xdpy, 0);
  data = xdo_get_window_property_by_atom(xdo, root, request, &nitems, &type, &size);

  if (type != XA_CARDINAL) {
    fprintf(stderr, 
            "Got unexpected type returned from _NET_DESKTOP_VIEWPORT."
            " Expected CARDINAL, got %s\n",
            XGetAtomName(xdo->xdpy, type));
    free(data);
    return XDO_ERROR;
  }

  if (nitems != 2) {
    fprintf(stderr, "Expected 2 items for _NET_DESKTOP_VIEWPORT, got %ld\n",
            nitems);
    free(data);
    return XDO_ERROR;
  }

  int *viewport_data = (int *)data;
  *x_ret = viewport_data[0];
  *y_ret = viewport_data[1];
  free(data);

  return XDO_SUCCESS;
}

int xdo_set_desktop_viewport(const xdo_t *xdo, int x, int y) {
  XEvent xev;
  int ret;
  Window root = RootWindow(xdo->xdpy, 0);

  memset(&xev, 0, sizeof(xev));
  xev.type = ClientMessage;
  xev.xclient.display = xdo->xdpy;
  xev.xclient.window = root;
  xev.xclient.message_type = XInternAtom(xdo->xdpy, "_NET_DESKTOP_VIEWPORT",
                                         False);
  xev.xclient.format = 32;
  xev.xclient.data.l[0] = x;
  xev.xclient.data.l[1] = y;

  ret = XSendEvent(xdo->xdpy, root, False,
                   SubstructureNotifyMask | SubstructureRedirectMask, &xev);

  /* XXX: XSendEvent returns 0 on conversion failure, nonzero otherwise.
   * Manpage says it will only generate BadWindow or BadValue errors */
  return _is_success("XSendEvent[EWMH:_NET_DESKTOP_VIEWPORT]", ret == 0, xdo);
}

int xdo_kill_window(const xdo_t *xdo, Window window) {
  int ret;
  ret = XKillClient(xdo->xdpy, window);
  return _is_success("XKillClient", ret == 0, xdo);
}

int xdo_close_window(const xdo_t *xdo, Window window) {
  int ret;
  ret = XDestroyWindow(xdo->xdpy, window);
  return _is_success("XDestroyWindow", ret == 0, xdo);
}

int xdo_quit_window(const xdo_t *xdo, Window window) {
  XEvent xev;
  int ret;
  Window root = RootWindow(xdo->xdpy, 0);

  memset(&xev, 0, sizeof(xev));
  xev.type = ClientMessage;
  xev.xclient.serial = 0;
  xev.xclient.send_event = True;
  xev.xclient.display = xdo->xdpy;
  xev.xclient.window = window;
  xev.xclient.message_type = XInternAtom(xdo->xdpy, "_NET_CLOSE_WINDOW", False);
  xev.xclient.format = 32;

  ret = XSendEvent(xdo->xdpy, root, False,
                   SubstructureNotifyMask | SubstructureRedirectMask,
                   &xev);

  /* XXX: XSendEvent returns 0 on conversion failure, nonzero otherwise.
   * Manpage says it will only generate BadWindow or BadValue errors */
  return _is_success("XSendEvent[_NET_CLOSE_WINDOW]", ret == 0, xdo);
}

int xdo_get_window_name(const xdo_t *xdo, Window window, 
                        unsigned char **name_ret, int *name_len_ret,
                        int *name_type) {
  if (atom_NET_WM_NAME == (Atom)-1) {
    atom_NET_WM_NAME = XInternAtom(xdo->xdpy, "_NET_WM_NAME", False);
  } 
  if (atom_WM_NAME == (Atom)-1) {
    atom_WM_NAME = XInternAtom(xdo->xdpy, "WM_NAME", False);
  }
  if (atom_STRING == (Atom)-1) {
    atom_STRING = XInternAtom(xdo->xdpy, "STRING", False);
  }
  if (atom_UTF8_STRING == (Atom)-1) {
    atom_UTF8_STRING = XInternAtom(xdo->xdpy, "UTF8_STRING", False);
  }

  Atom type;
  int size;
  long nitems;

  /**
   * http://standards.freedesktop.org/wm-spec/1.3/ar01s05.html
   * Prefer _NET_WM_NAME if available, otherwise use WM_NAME
   * If no WM_NAME, set name_ret to NULL and set len to 0
   */

  *name_ret = xdo_get_window_property_by_atom(xdo, window, atom_NET_WM_NAME, &nitems,
                             &type, &size);
  if (nitems == 0) {
    *name_ret = xdo_get_window_property_by_atom(xdo, window, atom_WM_NAME, &nitems,
                               &type, &size);
  }
  *name_len_ret = nitems;
  *name_type = type;

  return 0;
}

int xdo_get_window_classname(const xdo_t *xdo, Window window, unsigned char **class_ret) {
  XClassHint classhint;
  Status ret = XGetClassHint(xdo->xdpy, window, &classhint);

  if (ret) {
    XFree(classhint.res_name);
    *class_ret = (unsigned char*) classhint.res_class;
  } else {
    *class_ret = NULL;
  }
  return _is_success("XGetClassHint[WM_CLASS]", ret == 0, xdo);
}

int xdo_window_state(xdo_t *xdo, Window window, unsigned long action, const char *property) {
  int ret;
  XEvent xev;
  Window root = RootWindow(xdo->xdpy, 0);

  memset(&xev, 0, sizeof(xev));
  xev.xclient.type = ClientMessage;
  xev.xclient.serial = 0;
  xev.xclient.send_event = True;
  xev.xclient.message_type = XInternAtom(xdo->xdpy, "_NET_WM_STATE", False);
  xev.xclient.window = window;
  xev.xclient.format = 32;
  xev.xclient.data.l[0] = action;
  xev.xclient.data.l[1] = XInternAtom(xdo->xdpy, property, False);

  ret = XSendEvent(xdo->xdpy, root, False,
                   SubstructureNotifyMask | SubstructureRedirectMask, &xev);
  return _is_success("XSendEvent[EWMH:_NET_WM_STATE]", ret == 0, xdo);
}

int xdo_minimize_window(const xdo_t *xdo, Window window) {
  int ret;
  int screen;

  /* Get screen number */
  XWindowAttributes attr;
  XGetWindowAttributes(xdo->xdpy, window, &attr);
  screen = XScreenNumberOfScreen(attr.screen);

  /* Minimize it */
  ret = XIconifyWindow(xdo->xdpy, window, screen);
  return _is_success("XIconifyWindow", ret == 0, xdo);
}

void _xdo_debug(const xdo_t *xdo, const char *format, ...) {
  va_list args;

  va_start(args, format);
  if (xdo->debug) {
    vfprintf(stderr, format, args);
    fprintf(stderr, "\n");
  }
} /* _xdo_debug */

/* Used for printing things conditionally based on xdo->quiet */
void _xdo_eprintf(const xdo_t *xdo, int hushable, const char *format, ...) {
  va_list args;

  va_start(args, format);
  if (xdo->quiet == True && hushable) {
    return;
  }

  vfprintf(stderr, format, args);
  fprintf(stderr, "\n");
} /* _xdo_eprintf */

void xdo_enable_feature(xdo_t *xdo, int feature) {
  xdo->features_mask |= (0 << feature);
}

void xdo_disable_feature(xdo_t *xdo, int feature) {
  xdo->features_mask &= ~(1 << feature);
}

int xdo_has_feature(xdo_t *xdo, int feature) {
  return (xdo->features_mask & (1 << feature));
}

// Espanso-specific variants
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

        XFlush(xdo->xdpy);
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

int fast_send_keysequence_window_do(const xdo_t *xdo, Window window, const char *keyseq,
                        int pressed, int *modifier, useconds_t delay) {
  int ret = 0;
  charcodemap_t *keys = NULL;
  int nkeys = 0;

  if (_xdo_send_keysequence_window_to_keycode_list(xdo, keyseq, &keys, &nkeys) == False) {
    fprintf(stderr, "Failure converting key sequence '%s' to keycodes\n", keyseq);
    return 1;
  }

  ret = fast_send_keysequence_window_list_do(xdo, window, keys, nkeys, pressed, modifier, delay);
  free(keys);

  return ret;
}

int fast_send_keysequence_window(const xdo_t *xdo, Window window, const char *keyseq,
                    useconds_t delay) {
  int ret = 0;
  int modifier = 0;
  ret += fast_send_keysequence_window_do(xdo, window, keyseq, True, &modifier, delay / 2);
  ret += fast_send_keysequence_window_do(xdo, window, keyseq, False, &modifier, delay / 2);
  return ret;
}
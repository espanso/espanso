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

#include "native.h"
#include <X11/Xlibint.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/cursorfont.h>
#include <X11/keysymdef.h>
#include <X11/keysym.h>
#include <X11/XKBlib.h>
#include <X11/Xatom.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

// Function taken from the wmlib tool source code
char *get_property(Display *disp, Window win,
                   Atom xa_prop_type, char *prop_name, unsigned long *size)
{
  unsigned long ret_nitems, ret_bytes_after, tmp_size;
  Atom xa_prop_name, xa_ret_type;
  unsigned char *ret_prop;
  int ret_format;
  char *ret;
  int size_in_byte;

  xa_prop_name = XInternAtom(disp, prop_name, False);

  if (XGetWindowProperty(disp, win, xa_prop_name, 0, 4096 / 4, False,
                         xa_prop_type, &xa_ret_type, &ret_format, &ret_nitems,
                         &ret_bytes_after, &ret_prop) != Success)
    return NULL;

  if (xa_ret_type != xa_prop_type)
  {
    XFree(ret_prop);
    return NULL;
  }

  switch (ret_format)
  {
  case 8:
    size_in_byte = sizeof(char);
    break;
  case 16:
    size_in_byte = sizeof(short);
    break;
  case 32:
    size_in_byte = sizeof(long);
    break;
  }

  tmp_size = size_in_byte * ret_nitems;
  ret = (char *)malloc(tmp_size + 1);
  memcpy(ret, ret_prop, tmp_size);
  ret[tmp_size] = '\0';

  if (size)
    *size = tmp_size;

  XFree(ret_prop);
  return ret;
}

// Function taken from Window Management Library for Ruby
char *xwm_get_win_title(Display *disp, Window win)
{
  char *wname = (char *)get_property(disp, win, XA_STRING, "WM_NAME", NULL);
  char *nwname = (char *)get_property(disp, win, XInternAtom(disp, "UTF8_STRING", False), "_NET_WM_NAME", NULL);

  return nwname ? nwname : (wname ? wname : NULL);
}

int32_t info_get_title(char *buffer, int32_t buffer_size)
{
  Display *display = XOpenDisplay(0);

  if (!display)
  {
    return -1;
  }

  Window focused;
  int revert_to;
  int ret = XGetInputFocus(display, &focused, &revert_to);

  int result = 1;
  if (!ret)
  {
    fprintf(stderr, "xdo_get_active_window reported an error\n");
    result = -2;
  }
  else
  {
    char *title = xwm_get_win_title(display, focused);

    snprintf(buffer, buffer_size, "%s", title);

    XFree(title);
  }

  XCloseDisplay(display);

  return result;
}

int32_t info_get_exec(char *buffer, int32_t buffer_size)
{
  Display *display = XOpenDisplay(0);

  if (!display)
  {
    return -1;
  }

  Window focused;
  int revert_to;
  int ret = XGetInputFocus(display, &focused, &revert_to);

  int result = 1;
  if (!ret)
  {
    fprintf(stderr, "xdo_get_active_window reported an error\n");
    result = -2;
  }
  else
  {
    // Get the window process PID
    char *pid_raw = (char *)get_property(display, focused, XA_CARDINAL, "_NET_WM_PID", NULL);
    if (pid_raw == NULL)
    {
      result = -3;
    }
    else
    {
      int pid = pid_raw[0] | pid_raw[1] << 8 | pid_raw[2] << 16 | pid_raw[3] << 24;

      // Get the executable path from it
      char proc_path[250];
      snprintf(proc_path, 250, "/proc/%d/exe", pid);

      readlink(proc_path, buffer, buffer_size);

      XFree(pid_raw);
    }
  }

  XCloseDisplay(display);

  return result;
}

int32_t info_get_class(char *buffer, int32_t buffer_size)
{
  Display *display = XOpenDisplay(0);

  if (!display)
  {
    return -1;
  }

  Window focused;
  int revert_to;
  int ret = XGetInputFocus(display, &focused, &revert_to);

  int result = 1;
  if (!ret)
  {
    fprintf(stderr, "xdo_get_active_window reported an error\n");
    result = -2;
  }
  else
  {
    XClassHint hint;

    if (XGetClassHint(display, focused, &hint))
    {
      snprintf(buffer, buffer_size, "%s", hint.res_class);
      XFree(hint.res_name);
      XFree(hint.res_class);
    }
  }

  XCloseDisplay(display);

  return result;
}
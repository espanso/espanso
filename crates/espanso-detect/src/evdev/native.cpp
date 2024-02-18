// A good portion of the following code has been taken by the "interactive-evdev.c"
// example of "libxkbcommon" by Ran Benita. The original license is included as follows:
// https://github.com/xkbcommon/libxkbcommon/blob/master/tools/interactive-evdev.c

/*
 * Copyright © 2012 Ran Benita <ran234@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice (including the next
 * paragraph) shall be included in all copies or substantial portions of the
 * Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

#include "native.h"

#include <assert.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <fnmatch.h>
#include <getopt.h>
#include <limits.h>
#include <locale.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <string>

#include <sys/epoll.h>
#include <linux/input.h>

#include "xkbcommon/xkbcommon.h"

#define NLONGS(n) (((n) + LONG_BIT - 1) / LONG_BIT)

static bool
evdev_bit_is_set(const unsigned long *array, int bit)
{
  return array[bit / LONG_BIT] & (1LL << (bit % LONG_BIT));
}

/* Some heuristics to see if the device is a keyboard. */
int32_t is_keyboard_or_mouse(int fd)
{
  int i;
  unsigned long evbits[NLONGS(EV_CNT)] = {0};
  unsigned long keybits[NLONGS(KEY_CNT)] = {0};

  errno = 0;
  ioctl(fd, EVIOCGBIT(0, sizeof(evbits)), evbits);
  if (errno)
    return false;

  if (!evdev_bit_is_set(evbits, EV_KEY))
    return false;

  errno = 0;
  ioctl(fd, EVIOCGBIT(EV_KEY, sizeof(keybits)), keybits);
  if (errno)
    return false;

  // Test for keyboard keys
  for (i = KEY_RESERVED; i <= KEY_MIN_INTERESTING; i++)
    if (evdev_bit_is_set(keybits, i))
      return true;

  // Test for mouse keys
  for (i = BTN_MOUSE; i <= BTN_TASK; i++)
    if (evdev_bit_is_set(keybits, i))
      return true;

  return false;
}
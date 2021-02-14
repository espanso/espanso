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
#include <linux/uinput.h>
#include <memory.h>

unsigned long ui_dev_destroy()
{
  return UI_DEV_DESTROY;
}

unsigned long ui_dev_create()
{
  return UI_DEV_CREATE;
}

unsigned long ui_set_evbit()
{
  return UI_SET_EVBIT;
}

unsigned long ui_set_keybit()
{
  return UI_SET_KEYBIT;
}

int setup_uinput_device(int fd)
{
  struct uinput_setup usetup;

  memset(&usetup, 0, sizeof(usetup));
  usetup.id.bustype = BUS_USB;
  usetup.id.vendor = 0x1234;  // sample vendor
  usetup.id.product = 0x5678; // sample product
  strcpy(usetup.name, "Espanso virtual device");

  return ioctl(fd, UI_DEV_SETUP, &usetup);
}

void emit(int fd, int type, int code, int val)
{
   struct input_event ie;
   ie.type = type;
   ie.code = code;
   ie.value = val;
   // timestamp values below are ignored
   ie.time.tv_sec = 0;
   ie.time.tv_usec = 0;

   write(fd, &ie, sizeof(ie));
}

void uinput_emit(int fd, unsigned int code, int pressed) {
  emit(fd, EV_KEY, code, pressed);
  emit(fd, EV_SYN, SYN_REPORT, 0);
}
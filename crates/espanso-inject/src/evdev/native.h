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

#ifndef ESPANSO_INJECT_EV_CONSTANTS_H
#define ESPANSO_INJECT_EV_CONSTANTS_H

unsigned long ui_dev_destroy();
unsigned long ui_dev_create();
unsigned long ui_set_evbit();
unsigned long ui_set_keybit();

int setup_uinput_device(int fd);
void uinput_emit(int fd, unsigned int code, int pressed);

#endif //ESPANSO_INJECT_EV_CONSTANTS_H
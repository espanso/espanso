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
#include "clip/clip.h"
#include "string.h"
#include <iostream>

clip::format html_format = clip::register_format("text/html");
clip::format png_format = clip::register_format("image/png");

int32_t clipboard_x11_get_text(char * buffer, int32_t buffer_size) {
  std::string value;
  if (!clip::get_text(value)) {
    return 0;
  }

  if (value.length() == 0) {
    return 0;
  }

  strncpy(buffer, value.c_str(), buffer_size - 1);
  return 1;
}

int32_t clipboard_x11_set_text(char * text) {
  if (!clip::set_text(text)) {
    return 0;
  } else {
    return 1;
  }
}

int32_t clipboard_x11_set_html(char * html, char * fallback_text) {
  clip::lock l;
  if (!l.clear()) {
    return 0;
  }
  if (!l.set_data(html_format, html, strlen(html))) {
    return 0;
  }
  if (fallback_text) {
    // Best effort to set the fallback
    l.set_data(clip::text_format(), fallback_text, strlen(fallback_text));
  }
  return 1;
}

int32_t clipboard_x11_set_image(char * buffer, int32_t size) {
  clip::lock l;
  if (!l.clear()) {
    return 0;
  }

  if (!l.set_data(png_format, buffer, size)) {
    return 0;
  }

  return 1;
}
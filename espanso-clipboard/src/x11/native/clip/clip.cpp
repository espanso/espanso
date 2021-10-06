// Clip Library
// Copyright (c) 2015-2018 David Capello
//
// This file is released under the terms of the MIT license.
// Read LICENSE.txt for more information.

#include "clip.h"
#include "clip_lock_impl.h"

#include <vector>
#include <stdexcept>

namespace clip {

namespace {

void default_error_handler(ErrorCode code) {
  static const char* err[] = {
    "Cannot lock clipboard",
    "Image format is not supported"
  };
  throw std::runtime_error(err[static_cast<int>(code)]);
}

} // anonymous namespace

error_handler g_error_handler = default_error_handler;

lock::lock(void* native_window_handle)
  : p(new impl(native_window_handle)) {
}

lock::~lock() = default;

bool lock::locked() const {
  return p->locked();
}

bool lock::clear() {
  return p->clear();
}

bool lock::is_convertible(format f) const {
  return p->is_convertible(f);
}

bool lock::set_data(format f, const char* buf, size_t length) {
  return p->set_data(f, buf, length);
}

bool lock::get_data(format f, char* buf, size_t len) const {
  return p->get_data(f, buf, len);
}

size_t lock::get_data_length(format f) const {
  return p->get_data_length(f);
}

bool lock::set_image(const image& img) {
  return p->set_image(img);
}

bool lock::get_image(image& img) const {
  return p->get_image(img);
}

bool lock::get_image_spec(image_spec& spec) const {
  return p->get_image_spec(spec);
}

format empty_format() { return 0; }
format text_format()  { return 1; }
format image_format() { return 2; }

bool has(format f) {
  lock l;
  if (l.locked())
    return l.is_convertible(f);
  else
    return false;
}

bool clear() {
  lock l;
  if (l.locked())
    return l.clear();
  else
    return false;
}

bool set_text(const std::string& value) {
  lock l;
  if (l.locked()) {
    l.clear();
    return l.set_data(text_format(), value.c_str(), value.size());
  }
  else
    return false;
}

bool get_text(std::string& value) {
  lock l;
  if (!l.locked())
    return false;

  format f = text_format();
  if (!l.is_convertible(f))
    return false;

  size_t len = l.get_data_length(f);
  if (len > 0) {
    std::vector<char> buf(len);
    l.get_data(f, &buf[0], len);
    value = &buf[0];
    return true;
  }
  else {
    value.clear();
    return true;
  }
}

bool set_image(const image& img) {
  lock l;
  if (l.locked()) {
    l.clear();
    return l.set_image(img);
  }
  else
    return false;
}

bool get_image(image& img) {
  lock l;
  if (!l.locked())
    return false;

  format f = image_format();
  if (!l.is_convertible(f))
    return false;

  return l.get_image(img);
}

bool get_image_spec(image_spec& spec) {
  lock l;
  if (!l.locked())
    return false;

  format f = image_format();
  if (!l.is_convertible(f))
    return false;

  return l.get_image_spec(spec);
}

void set_error_handler(error_handler handler) {
  g_error_handler = handler;
}

error_handler get_error_handler() {
  return g_error_handler;
}

#ifdef HAVE_XCB_XLIB_H
static int g_x11_timeout = 1000;
void set_x11_wait_timeout(int msecs) { g_x11_timeout = msecs; }
int get_x11_wait_timeout() { return g_x11_timeout; }
#else
void set_x11_wait_timeout(int) { }
int get_x11_wait_timeout() { return 1000; }
#endif

} // namespace clip

// Clip Library
// Copyright (c) 2015-2018 David Capello
//
// This file is released under the terms of the MIT license.
// Read LICENSE.txt for more information.

#ifndef CLIP_LOCK_IMPL_H_INCLUDED
#define CLIP_LOCK_IMPL_H_INCLUDED

namespace clip {

class lock::impl {
public:
  impl(void* native_window_handle);
  ~impl();

  bool locked() const { return m_locked; }
  bool clear();
  bool is_convertible(format f) const;
  bool set_data(format f, const char* buf, size_t len);
  bool get_data(format f, char* buf, size_t len) const;
  size_t get_data_length(format f) const;
  bool set_image(const image& image);
  bool get_image(image& image) const;
  bool get_image_spec(image_spec& spec) const;

private:
  bool m_locked;
};

} // namespace clip

#endif

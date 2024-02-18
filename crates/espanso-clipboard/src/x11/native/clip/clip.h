// Clip Library
// Copyright (c) 2015-2018 David Capello
//
// This file is released under the terms of the MIT license.
// Read LICENSE.txt for more information.

#ifndef CLIP_H_INCLUDED
#define CLIP_H_INCLUDED
#pragma once

#include <cassert>
#include <memory>
#include <string>

namespace clip {

  // ======================================================================
  // Low-level API to lock the clipboard/pasteboard and modify it
  // ======================================================================

  // Clipboard format identifier.
  typedef size_t format;

  class image;
  struct image_spec;

  class lock {
  public:
    // You can give your current HWND as the "native_window_handle."
    // Windows clipboard functions use this handle to open/close
    // (lock/unlock) the clipboard. From the MSDN documentation we
    // need this handler so SetClipboardData() doesn't fail after a
    // EmptyClipboard() call. Anyway it looks to work just fine if we
    // call OpenClipboard() with a null HWND.
    lock(void* native_window_handle = nullptr);
    ~lock();

    // Returns true if we've locked the clipboard successfully in
    // lock() constructor.
    bool locked() const;

    // Clears the clipboard content. If you don't clear the content,
    // previous clipboard content (in unknown formats) could persist
    // after the unlock.
    bool clear();

    // Returns true if the clipboard can be converted to the given
    // format.
    bool is_convertible(format f) const;
    bool set_data(format f, const char* buf, size_t len);
    bool get_data(format f, char* buf, size_t len) const;
    size_t get_data_length(format f) const;

    // For images
    bool set_image(const image& image);
    bool get_image(image& image) const;
    bool get_image_spec(image_spec& spec) const;

  private:
    class impl;
    std::unique_ptr<impl> p;
  };

  format register_format(const std::string& name);

  // This format is when the clipboard has no content.
  format empty_format();

  // When the clipboard has UTF8 text.
  format text_format();

  // When the clipboard has an image.
  format image_format();

  // Returns true if the clipboard has content of the given type.
  bool has(format f);

  // Clears the clipboard content.
  bool clear();

  // ======================================================================
  // Error handling
  // ======================================================================

  enum class ErrorCode {
    CannotLock,
    ImageNotSupported,
  };

  typedef void (*error_handler)(ErrorCode code);

  void set_error_handler(error_handler f);
  error_handler get_error_handler();

  // ======================================================================
  // Text
  // ======================================================================

  // High-level API to put/get UTF8 text in/from the clipboard. These
  // functions returns false in case of error.
  bool set_text(const std::string& value);
  bool get_text(std::string& value);

  // ======================================================================
  // Image
  // ======================================================================

  struct image_spec {
    unsigned long width = 0;
    unsigned long height = 0;
    unsigned long bits_per_pixel = 0;
    unsigned long bytes_per_row = 0;
    unsigned long red_mask = 0;
    unsigned long green_mask = 0;
    unsigned long blue_mask = 0;
    unsigned long alpha_mask = 0;
    unsigned long red_shift = 0;
    unsigned long green_shift = 0;
    unsigned long blue_shift = 0;
    unsigned long alpha_shift = 0;
  };

  // The image data must contain straight RGB values
  // (non-premultiplied by alpha). The image retrieved from the
  // clipboard will be non-premultiplied too. Basically you will be
  // always dealing with straight alpha images.
  //
  // Details: Windows expects premultiplied images on its clipboard
  // content, so the library code make the proper conversion
  // automatically. macOS handles straight alpha directly, so there is
  // no conversion at all. Linux/X11 images are transferred in
  // image/png format which are specified in straight alpha.
  class image {
  public:
    image();
    image(const image_spec& spec);
    image(const void* data, const image_spec& spec);
    image(const image& image);
    image(image&& image);
    ~image();

    image& operator=(const image& image);
    image& operator=(image&& image);

    char* data() const { return m_data; }
    const image_spec& spec() const { return m_spec; }

    bool is_valid() const { return m_data != nullptr; }
    void reset();

  private:
    void copy_image(const image& image);
    void move_image(image&& image);

    bool m_own_data;
    char* m_data;
    image_spec m_spec;
  };

  // High-level API to set/get an image in/from the clipboard. These
  // functions returns false in case of error.
  bool set_image(const image& img);
  bool get_image(image& img);
  bool get_image_spec(image_spec& spec);

  // ======================================================================
  // Platform-specific
  // ======================================================================

  // Only for X11: Sets the time (in milliseconds) that we must wait
  // for the selection/clipboard owner to receive the content. This
  // value is 1000 (one second) by default.
  void set_x11_wait_timeout(int msecs);
  int get_x11_wait_timeout();

} // namespace clip

#endif // CLIP_H_INCLUDED

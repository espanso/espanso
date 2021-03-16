// Clip Library
// Copyright (c) 2018-2019 David Capello
//
// This file is released under the terms of the MIT license.
// Read LICENSE.txt for more information.

#include "clip.h"
#include "clip_lock_impl.h"

#include <xcb/xcb.h>

#include <algorithm>
#include <cassert>
#include <condition_variable>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <functional>
#include <map>
#include <memory>
#include <mutex>
#include <thread>
#include <vector>

#ifdef HAVE_PNG_H
  #include "clip_x11_png.h"
#endif

#define CLIP_SUPPORT_SAVE_TARGETS 1

namespace clip {

namespace {

enum CommonAtom {
  ATOM,
  INCR,
  TARGETS,
  CLIPBOARD,
#ifdef HAVE_PNG_H
  MIME_IMAGE_PNG,
#endif
#ifdef CLIP_SUPPORT_SAVE_TARGETS
  ATOM_PAIR,
  SAVE_TARGETS,
  MULTIPLE,
  CLIPBOARD_MANAGER,
#endif
};

const char* kCommonAtomNames[] = {
  "ATOM",
  "INCR",
  "TARGETS",
  "CLIPBOARD",
#ifdef HAVE_PNG_H
  "image/png",
#endif
#ifdef CLIP_SUPPORT_SAVE_TARGETS
  "ATOM_PAIR",
  "SAVE_TARGETS",
  "MULTIPLE",
  "CLIPBOARD_MANAGER",
#endif
};

const int kBaseForCustomFormats = 100;

class Manager {
public:
  typedef std::shared_ptr<std::vector<uint8_t>> buffer_ptr;
  typedef std::vector<xcb_atom_t> atoms;
  typedef std::function<bool()> notify_callback;

  Manager()
    : m_lock(m_mutex, std::defer_lock)
    , m_connection(xcb_connect(nullptr, nullptr))
    , m_window(0)
    , m_incr_process(false) {
    if (!m_connection)
      return;

    const xcb_setup_t* setup = xcb_get_setup(m_connection);
    if (!setup)
      return;

    xcb_screen_t* screen = xcb_setup_roots_iterator(setup).data;
    if (!screen)
      return;

    uint32_t event_mask =
      // Just in case that some program reports SelectionNotify events
      // with XCB_EVENT_MASK_PROPERTY_CHANGE mask.
      XCB_EVENT_MASK_PROPERTY_CHANGE |
      // To receive DestroyNotify event and stop the message loop.
      XCB_EVENT_MASK_STRUCTURE_NOTIFY;

    m_window = xcb_generate_id(m_connection);
    xcb_create_window(m_connection, 0,
                      m_window,
                      screen->root,
                      0, 0, 1, 1, 0,
                      XCB_WINDOW_CLASS_INPUT_OUTPUT,
                      screen->root_visual,
                      XCB_CW_EVENT_MASK,
                      &event_mask);

    m_thread = std::thread(
      [this]{
        process_x11_events();
      });
  }

  ~Manager() {
#ifdef CLIP_SUPPORT_SAVE_TARGETS
    if (!m_data.empty() &&
        m_window &&
        m_window == get_x11_selection_owner()) {
      // Check if there is a CLIPBOARD_MANAGER running to save all
      // targets before we exit.
      xcb_window_t x11_clipboard_manager = 0;
      xcb_get_selection_owner_cookie_t cookie =
        xcb_get_selection_owner(m_connection,
                                get_atom(CLIPBOARD_MANAGER));

      xcb_get_selection_owner_reply_t* reply =
        xcb_get_selection_owner_reply(m_connection, cookie, nullptr);
      if (reply) {
        x11_clipboard_manager = reply->owner;
        free(reply);
      }

      if (x11_clipboard_manager) {
        // Start the SAVE_TARGETS mechanism so the X11
        // CLIPBOARD_MANAGER will save our clipboard data
        // from now on.
        get_data_from_selection_owner(
          { get_atom(SAVE_TARGETS) },
          [this]() -> bool { return true; },
          get_atom(CLIPBOARD_MANAGER));
      }
    }
#endif

    if (m_window) {
      xcb_destroy_window(m_connection, m_window);
      xcb_flush(m_connection);
    }

    if (m_thread.joinable())
      m_thread.join();

    if (m_connection)
      xcb_disconnect(m_connection);
  }

  bool try_lock() {
    bool res = m_lock.try_lock();
    if (!res) {
      // TODO make this configurable (the same for Windows retries)
      for (int i=0; i<5 && !res; ++i) {
        res = m_lock.try_lock();
        std::this_thread::sleep_for(std::chrono::milliseconds(20));
      }
    }
    return res;
  }

  void unlock() {
    m_lock.unlock();
  }

  // Clear our data
  void clear_data() {
    m_data.clear();
    m_image.reset();
  }

  void clear() {
    clear_data();

    // Clear the clipboard data from the selection owner
    const xcb_window_t owner = get_x11_selection_owner();
    if (m_window != owner) {
      xcb_selection_clear_event_t event;
      event.response_type = XCB_SELECTION_CLEAR;
      event.pad0          = 0;
      event.sequence      = 0;
      event.time          = XCB_CURRENT_TIME;
      event.owner         = owner;
      event.selection     = get_atom(CLIPBOARD);

      xcb_send_event(m_connection, false,
                     owner,
                     XCB_EVENT_MASK_NO_EVENT,
                     (const char*)&event);

      xcb_flush(m_connection);
    }
  }

  bool is_convertible(format f) const {
    const atoms atoms = get_format_atoms(f);
    const xcb_window_t owner = get_x11_selection_owner();

    // If we are the owner, we just can check the m_data map
    if (owner == m_window) {
      for (xcb_atom_t atom : atoms) {
        auto it = m_data.find(atom);
        if (it != m_data.end())
          return true;
      }
    }
    // Ask to the selection owner the available formats/atoms/targets.
    else if (owner) {
      return
        get_data_from_selection_owner(
          { get_atom(TARGETS) },
          [this, &atoms]() -> bool {
            assert(m_reply_data);
            if (!m_reply_data)
              return false;

            const xcb_atom_t* sel_atoms = (const xcb_atom_t*)&(*m_reply_data)[0];
            int sel_natoms = m_reply_data->size() / sizeof(xcb_atom_t);
            auto atoms_begin = atoms.begin();
            auto atoms_end = atoms.end();
            for (int i=0; i<sel_natoms; ++i) {
              if (std::find(atoms_begin,
                            atoms_end,
                            sel_atoms[i]) != atoms_end) {
                return true;
              }
            }
            return false;
          });
    }

    return false;
  }

  bool set_data(format f, const char* buf, size_t len) {
    if (!set_x11_selection_owner())
      return false;

    const atoms atoms = get_format_atoms(f);
    if (atoms.empty())
      return false;

    buffer_ptr shared_data_buf = std::make_shared<std::vector<uint8_t>>(len);
    std::copy(buf,
              buf+len,
              shared_data_buf->begin());
    for (xcb_atom_t atom : atoms)
      m_data[atom] = shared_data_buf;

    return true;
  }

  bool get_data(format f, char* buf, size_t len) const {
    const atoms atoms = get_format_atoms(f);
    const xcb_window_t owner = get_x11_selection_owner();
    if (owner == m_window) {
      for (xcb_atom_t atom : atoms) {
        auto it = m_data.find(atom);
        if (it != m_data.end()) {
          size_t n = std::min(len, it->second->size());
          std::copy(it->second->begin(),
                    it->second->begin()+n,
                    buf);

          if (f == text_format()) {
            // Add an extra null char
            if (n < len)
              buf[n] = 0;
          }

          return true;
        }
      }
    }
    else if (owner) {
      if (get_data_from_selection_owner(
            atoms,
            [this, buf, len, f]() -> bool {
              size_t n = std::min(len, m_reply_data->size());
              std::copy(m_reply_data->begin(),
                        m_reply_data->begin()+n,
                        buf);

              if (f == text_format()) {
                if (n < len)
                  buf[n] = 0; // Include a null character
              }

              return true;
            })) {
        return true;
      }
    }
    return false;
  }

  size_t get_data_length(format f) const {
    size_t len = 0;
    const atoms atoms = get_format_atoms(f);
    const xcb_window_t owner = get_x11_selection_owner();
    if (owner == m_window) {
      for (xcb_atom_t atom : atoms) {
        auto it = m_data.find(atom);
        if (it != m_data.end()) {
          len = it->second->size();
          break;
        }
      }
    }
    else if (owner) {
      if (!get_data_from_selection_owner(
            atoms,
            [this, &len]() -> bool {
              len = m_reply_data->size();
              return true;
            })) {
        // Error getting data length
        return 0;
      }
    }
    if (f == text_format() && len > 0) {
      ++len; // Add an extra byte for the null char
    }
    return len;
  }

  bool set_image(const image& image) {
    if (!set_x11_selection_owner())
      return false;

    m_image = image;

#ifdef HAVE_PNG_H
    // Put a nullptr in the m_data for image/png format and then we'll
    // encode the png data when the image is requested in this format.
    m_data[get_atom(MIME_IMAGE_PNG)] = buffer_ptr();
#endif

    return true;
  }

  bool get_image(image& output_img) const {
    const xcb_window_t owner = get_x11_selection_owner();
    if (owner == m_window) {
      if (m_image.is_valid()) {
        output_img = m_image;
        return true;
      }
    }
#ifdef HAVE_PNG_H
    else if (owner &&
             get_data_from_selection_owner(
               { get_atom(MIME_IMAGE_PNG) },
               [this, &output_img]() -> bool {
                 return x11::read_png(&(*m_reply_data)[0],
                                      m_reply_data->size(),
                                      &output_img, nullptr);
               })) {
      return true;
    }
#endif
    return false;
  }

  bool get_image_spec(image_spec& spec) const {
    const xcb_window_t owner = get_x11_selection_owner();
    if (owner == m_window) {
      if (m_image.is_valid()) {
        spec = m_image.spec();
        return true;
      }
    }
#ifdef HAVE_PNG_H
    else if (owner &&
             get_data_from_selection_owner(
               { get_atom(MIME_IMAGE_PNG) },
               [this, &spec]() -> bool {
                 return x11::read_png(&(*m_reply_data)[0],
                                      m_reply_data->size(),
                                      nullptr, &spec);
               })) {
      return true;
    }
#endif
    return false;
  }

  format register_format(const std::string& name) {
    xcb_atom_t atom = get_atom(name.c_str());
    m_custom_formats.push_back(atom);
    return (format)(m_custom_formats.size()-1) + kBaseForCustomFormats;
  }

private:

  void process_x11_events() {
    bool stop = false;
    xcb_generic_event_t* event;
    while (!stop && (event = xcb_wait_for_event(m_connection))) {
      int type = (event->response_type & ~0x80);

      switch (type) {

        case XCB_DESTROY_NOTIFY:
          // To stop the message loop we can just destroy the window
          stop = true;
          break;

        // Someone else has new content in the clipboard, so is
        // notifying us that we should delete our data now.
        case XCB_SELECTION_CLEAR:
          handle_selection_clear_event(
            (xcb_selection_clear_event_t*)event);
          break;

          // Someone is requesting the clipboard content from us.
        case XCB_SELECTION_REQUEST:
          handle_selection_request_event(
            (xcb_selection_request_event_t*)event);
          break;

          // We've requested the clipboard content and this is the
          // answer.
        case XCB_SELECTION_NOTIFY:
          handle_selection_notify_event(
            (xcb_selection_notify_event_t*)event);
          break;

        case XCB_PROPERTY_NOTIFY:
          handle_property_notify_event(
            (xcb_property_notify_event_t*)event);
          break;

      }

      free(event);
    }
  }

  void handle_selection_clear_event(xcb_selection_clear_event_t* event) {
    if (event->selection == get_atom(CLIPBOARD)) {
      std::lock_guard<std::mutex> lock(m_mutex);
      clear_data(); // Clear our clipboard data
    }
  }

  void handle_selection_request_event(xcb_selection_request_event_t* event) {
    std::lock_guard<std::mutex> lock(m_mutex);

    if (event->target == get_atom(TARGETS)) {
      atoms targets;
      targets.push_back(get_atom(TARGETS));
#ifdef CLIP_SUPPORT_SAVE_TARGETS
      targets.push_back(get_atom(SAVE_TARGETS));
      targets.push_back(get_atom(MULTIPLE));
#endif
      for (const auto& it : m_data)
        targets.push_back(it.first);

      // Set the "property" of "requestor" with the clipboard
      // formats ("targets", atoms) that we provide.
      xcb_change_property(
        m_connection,
        XCB_PROP_MODE_REPLACE,
        event->requestor,
        event->property,
        get_atom(ATOM),
        8*sizeof(xcb_atom_t),
        targets.size(),
        &targets[0]);
    }
#ifdef CLIP_SUPPORT_SAVE_TARGETS
    else if (event->target == get_atom(SAVE_TARGETS)) {
      // Do nothing
    }
    else if (event->target == get_atom(MULTIPLE)) {
      xcb_get_property_reply_t* reply =
        get_and_delete_property(event->requestor,
                                event->property,
                                get_atom(ATOM_PAIR),
                                false);
      if (reply) {
        for (xcb_atom_t
               *ptr=(xcb_atom_t*)xcb_get_property_value(reply),
               *end=ptr + (xcb_get_property_value_length(reply)/sizeof(xcb_atom_t));
             ptr<end; ) {
          xcb_atom_t target = *ptr++;
          xcb_atom_t property = *ptr++;

          if (!set_requestor_property_with_clipboard_content(
                event->requestor,
                property,
                target)) {
            xcb_change_property(
              m_connection,
              XCB_PROP_MODE_REPLACE,
              event->requestor,
              event->property,
              XCB_ATOM_NONE, 0, 0, nullptr);
          }
        }

        free(reply);
      }
    }
#endif // CLIP_SUPPORT_SAVE_TARGETS
    else {
      if (!set_requestor_property_with_clipboard_content(
            event->requestor,
            event->property,
            event->target)) {
        return;
      }
    }

    // Notify the "requestor" that we've already updated the property.
    xcb_selection_notify_event_t notify;
    notify.response_type = XCB_SELECTION_NOTIFY;
    notify.pad0          = 0;
    notify.sequence      = 0;
    notify.time          = event->time;
    notify.requestor     = event->requestor;
    notify.selection     = event->selection;
    notify.target        = event->target;
    notify.property      = event->property;

    xcb_send_event(m_connection, false,
                   event->requestor,
                   XCB_EVENT_MASK_NO_EVENT, // SelectionNotify events go without mask
                   (const char*)&notify);

    xcb_flush(m_connection);
  }

  bool set_requestor_property_with_clipboard_content(const xcb_atom_t requestor,
                                                     const xcb_atom_t property,
                                                     const xcb_atom_t target) {
    auto it = m_data.find(target);
    if (it == m_data.end()) {
      // Nothing to do (unsupported target)
      return false;
    }

    // This can be null of the data was set from an image but we
    // didn't encode the image yet (e.g. to image/png format).
    if (!it->second) {
      encode_data_on_demand(*it);

      // Return nothing, the given "target" cannot be constructed
      // (maybe by some encoding error).
      if (!it->second)
        return false;
    }

    // Set the "property" of "requestor" with the
    // clipboard content in the requested format ("target").
    xcb_change_property(
      m_connection,
      XCB_PROP_MODE_REPLACE,
      requestor,
      property,
      target,
      8,
      it->second->size(),
      &(*it->second)[0]);
    return true;
  }

  void handle_selection_notify_event(xcb_selection_notify_event_t* event) {
    assert(event->requestor == m_window);

    if (event->target == get_atom(TARGETS))
      m_target_atom = get_atom(ATOM);
    else
      m_target_atom = event->target;

    xcb_get_property_reply_t* reply =
      get_and_delete_property(event->requestor,
                              event->property,
                              m_target_atom);
    if (reply) {
      // In this case, We're going to receive the clipboard content in
      // chunks of data with several PropertyNotify events.
      if (reply->type == get_atom(INCR)) {
        free(reply);

        reply = get_and_delete_property(event->requestor,
                                        event->property,
                                        get_atom(INCR));
        if (reply) {
          if (xcb_get_property_value_length(reply) == 4) {
            uint32_t n = *(uint32_t*)xcb_get_property_value(reply);
            m_reply_data = std::make_shared<std::vector<uint8_t>>(n);
            m_reply_offset = 0;
            m_incr_process = true;
            m_incr_received = true;
          }
          free(reply);
        }
      }
      else {
        // Simple case, the whole clipboard content in just one reply
        // (without the INCR method).
        m_reply_data.reset();
        m_reply_offset = 0;
        copy_reply_data(reply);

        call_callback(reply);

        free(reply);
      }
    }
  }

  void handle_property_notify_event(xcb_property_notify_event_t* event) {
    if (m_incr_process &&
        event->state == XCB_PROPERTY_NEW_VALUE &&
        event->atom == get_atom(CLIPBOARD)) {
      xcb_get_property_reply_t* reply =
        get_and_delete_property(event->window,
                                event->atom,
                                m_target_atom);
      if (reply) {
        m_incr_received = true;

        // When the length is 0 it means that the content was
        // completely sent by the selection owner.
        if (xcb_get_property_value_length(reply) > 0) {
          copy_reply_data(reply);
        }
        else {
          // Now that m_reply_data has the complete clipboard content,
          // we can call the m_callback.
          call_callback(reply);
          m_incr_process = false;
        }
        free(reply);
      }
    }
  }

  xcb_get_property_reply_t* get_and_delete_property(xcb_window_t window,
                                                    xcb_atom_t property,
                                                    xcb_atom_t atom,
                                                    bool delete_prop = true) {
    xcb_get_property_cookie_t cookie =
      xcb_get_property(m_connection,
                       delete_prop,
                       window,
                       property,
                       atom,
                       0, 0x1fffffff); // 0x1fffffff = INT32_MAX / 4

    xcb_generic_error_t* err = nullptr;
    xcb_get_property_reply_t* reply =
      xcb_get_property_reply(m_connection, cookie, &err);
    if (err) {
      // TODO report error
      free(err);
    }
    return reply;
  }

  // Concatenates the new data received in "reply" into "m_reply_data"
  // buffer.
  void copy_reply_data(xcb_get_property_reply_t* reply) {
    const uint8_t* src = (const uint8_t*)xcb_get_property_value(reply);
    // n = length of "src" in bytes
    size_t n = xcb_get_property_value_length(reply);

    size_t req = m_reply_offset+n;
    if (!m_reply_data) {
      m_reply_data = std::make_shared<std::vector<uint8_t>>(req);
    }
    // The "m_reply_data" size can be smaller because the size
    // specified in INCR property is just a lower bound.
    else if (req > m_reply_data->size()) {
      m_reply_data->resize(req);
    }

    std::copy(src, src+n, m_reply_data->begin()+m_reply_offset);
    m_reply_offset += n;
  }

  // Calls the current m_callback() to handle the clipboard content
  // received from the owner.
  void call_callback(xcb_get_property_reply_t* reply) {
    m_callback_result = false;
    if (m_callback)
      m_callback_result = m_callback();

    m_cv.notify_one();

    m_reply_data.reset();
  }

  bool get_data_from_selection_owner(const atoms& atoms,
                                     const notify_callback&& callback,
                                     xcb_atom_t selection = 0) const {
    if (!selection)
      selection = get_atom(CLIPBOARD);

    // Put the callback on "m_callback" so we can call it on
    // SelectionNotify event.
    m_callback = std::move(callback);

    // Clear data if we are not the selection owner.
    if (m_window != get_x11_selection_owner())
      m_data.clear();

    // Ask to the selection owner for its content on each known
    // text format/atom.
    for (xcb_atom_t atom : atoms) {
      xcb_convert_selection(m_connection,
                            m_window, // Send us the result
                            selection, // Clipboard selection
                            atom, // The clipboard format that we're requesting
                            get_atom(CLIPBOARD), // Leave result in this window's property
                            XCB_CURRENT_TIME);

      xcb_flush(m_connection);

      // We use the "m_incr_received" to wait several timeouts in case
      // that we've received the INCR SelectionNotify or
      // PropertyNotify events.
      do {
        m_incr_received = false;

        // Wait a response for 100 milliseconds
        std::cv_status status =
          m_cv.wait_for(m_lock,
                        std::chrono::milliseconds(get_x11_wait_timeout()));
        if (status == std::cv_status::no_timeout) {
          // If the condition variable was notified, it means that the
          // callback was called correctly.
          return m_callback_result;
        }
      } while (m_incr_received);
    }

    // Reset callback
    m_callback = notify_callback();
    return false;
  }

  atoms get_atoms(const char** names,
                  const int n) const {
    atoms result(n, 0);
    std::vector<xcb_intern_atom_cookie_t> cookies(n);

    for (int i=0; i<n; ++i) {
      auto it = m_atoms.find(names[i]);
      if (it != m_atoms.end())
        result[i] = it->second;
      else
        cookies[i] = xcb_intern_atom(
          m_connection, 0,
          std::strlen(names[i]), names[i]);
    }

    for (int i=0; i<n; ++i) {
      if (result[i] == 0) {
        xcb_intern_atom_reply_t* reply =
          xcb_intern_atom_reply(m_connection,
                                cookies[i],
                                nullptr);
        if (reply) {
          result[i] = m_atoms[names[i]] = reply->atom;
          free(reply);
        }
      }
    }

    return result;
  }

  xcb_atom_t get_atom(const char* name) const {
    auto it = m_atoms.find(name);
    if (it != m_atoms.end())
      return it->second;

    xcb_atom_t result = 0;
    xcb_intern_atom_cookie_t cookie =
      xcb_intern_atom(m_connection, 0,
                      std::strlen(name), name);

    xcb_intern_atom_reply_t* reply =
      xcb_intern_atom_reply(m_connection,
                            cookie,
                            nullptr);
    if (reply) {
      result = m_atoms[name] = reply->atom;
      free(reply);
    }
    return result;
  }

  xcb_atom_t get_atom(CommonAtom i) const {
    if (m_common_atoms.empty()) {
      m_common_atoms =
        get_atoms(kCommonAtomNames,
                  sizeof(kCommonAtomNames) / sizeof(kCommonAtomNames[0]));
    }
    return m_common_atoms[i];
  }

  const atoms& get_text_format_atoms() const {
    if (m_text_atoms.empty()) {
      const char* names[] = {
        // Prefer utf-8 formats first
        "UTF8_STRING",
        "text/plain;charset=utf-8",
        "text/plain;charset=UTF-8",
        // ANSI C strings?
        "STRING",
        "TEXT",
        "text/plain",
      };
      m_text_atoms = get_atoms(names, sizeof(names) / sizeof(names[0]));
    }
    return m_text_atoms;
  }

  const atoms& get_image_format_atoms() const {
    if (m_image_atoms.empty()) {
#ifdef HAVE_PNG_H
      m_image_atoms.push_back(get_atom(MIME_IMAGE_PNG));
#endif
    }
    return m_image_atoms;
  }

  atoms get_format_atoms(const format f) const {
    atoms atoms;
    if (f == text_format()) {
      atoms = get_text_format_atoms();
    }
    else if (f == image_format()) {
      atoms = get_image_format_atoms();
    }
    else {
      xcb_atom_t atom = get_format_atom(f);
      if (atom)
        atoms.push_back(atom);
    }
    return atoms;
  }

#if !defined(NDEBUG)
  // This can be used to print debugging messages.
  std::string get_atom_name(xcb_atom_t atom) const {
    std::string result;
    xcb_get_atom_name_cookie_t cookie =
      xcb_get_atom_name(m_connection, atom);
    xcb_generic_error_t* err = nullptr;
    xcb_get_atom_name_reply_t* reply =
      xcb_get_atom_name_reply(m_connection, cookie, &err);
    if (err) {
      free(err);
    }
    if (reply) {
      int len = xcb_get_atom_name_name_length(reply);
      if (len > 0) {
        result.resize(len);
        char* name = xcb_get_atom_name_name(reply);
        if (name)
          std::copy(name, name+len, result.begin());
      }
      free(reply);
    }
    return result;
  }
#endif

  bool set_x11_selection_owner() const {
    xcb_void_cookie_t cookie =
      xcb_set_selection_owner_checked(m_connection,
                                      m_window,
                                      get_atom(CLIPBOARD),
                                      XCB_CURRENT_TIME);
    xcb_generic_error_t* err =
      xcb_request_check(m_connection,
                        cookie);
    if (err) {
      free(err);
      return false;
    }
    return true;
  }

  xcb_window_t get_x11_selection_owner() const {
    xcb_window_t result = 0;
    xcb_get_selection_owner_cookie_t cookie =
      xcb_get_selection_owner(m_connection,
                              get_atom(CLIPBOARD));

    xcb_get_selection_owner_reply_t* reply =
      xcb_get_selection_owner_reply(m_connection, cookie, nullptr);
    if (reply) {
      result = reply->owner;
      free(reply);
    }
    return result;
  }

  xcb_atom_t get_format_atom(const format f) const {
    int i = f - kBaseForCustomFormats;
    if (i >= 0 && i < int(m_custom_formats.size()))
      return m_custom_formats[i];
    else
      return 0;
  }

  void encode_data_on_demand(std::pair<const xcb_atom_t, buffer_ptr>& e) {
#ifdef HAVE_PNG_H
    if (e.first == get_atom(MIME_IMAGE_PNG)) {
      assert(m_image.is_valid());
      if (!m_image.is_valid())
        return;

      std::vector<uint8_t> output;
      if (x11::write_png(m_image, output)) {
        e.second =
          std::make_shared<std::vector<uint8_t>>(
            std::move(output));
      }
      // else { TODO report png conversion errors }
    }
#endif
  }

  // Access to the whole Manager
  std::mutex m_mutex;

  // Lock used in the main thread using the Manager (i.e. by lock::impl)
  mutable std::unique_lock<std::mutex> m_lock;

  // Connection to X11 server
  xcb_connection_t* m_connection;

  // Temporal background window used to own the clipboard and process
  // all events related about the clipboard in a background thread
  xcb_window_t m_window;

  // Used to wait/notify the arrival of the SelectionNotify event when
  // we requested the clipboard content from other selection owner.
  mutable std::condition_variable m_cv;

  // Thread used to run a background message loop to wait X11 events
  // about clipboard. The X11 selection owner will be a hidden window
  // created by us just for the clipboard purpose/communication.
  std::thread m_thread;

  // Internal callback used when a SelectionNotify is received (or the
  // whole data content is received by the INCR method). So this
  // callback can use the notification by different purposes (e.g. get
  // the data length only, or get/process the data content, etc.).
  mutable notify_callback m_callback;

  // Result returned by the m_callback. Used as return value in the
  // get_data_from_selection_owner() function. For example, if the
  // callback must read a "image/png" file from the clipboard data and
  // fails, the callback can return false and finally the get_image()
  // will return false (i.e. there is data, but it's not a valid image
  // format).
  bool m_callback_result;

  // Cache of known atoms
  mutable std::map<std::string, xcb_atom_t> m_atoms;

  // Cache of common used atoms by us
  mutable atoms m_common_atoms;

  // Cache of atoms related to text or image content
  mutable atoms m_text_atoms;
  mutable atoms m_image_atoms;

  // Actual clipboard data generated by us (when we "copy" content in
  // the clipboard, it means that we own the X11 "CLIPBOARD"
  // selection, and in case of SelectionRequest events, we've to
  // return the data stored in this "m_data" field)
  mutable std::map<xcb_atom_t, buffer_ptr> m_data;

  // Copied image in the clipboard. As we have to transfer the image
  // in some specific format (e.g. image/png) we want to keep a copy
  // of the image and make the conversion when the clipboard data is
  // requested by other process.
  mutable image m_image;

  // True if we have received an INCR notification so we're going to
  // process several PropertyNotify to concatenate all data chunks.
  bool m_incr_process;

  // Variable used to wait more time if we've received an INCR
  // notification, which means that we're going to receive large
  // amounts of data from the selection owner.
  mutable bool m_incr_received;

  // Target/selection format used in the SelectionNotify. Used in the
  // INCR method to get data from the same property in the same format
  // (target) on each PropertyNotify.
  xcb_atom_t m_target_atom;

  // Each time we receive data from the selection owner, we put that
  // data in this buffer. If we get the data with the INCR method,
  // we'll concatenate chunks of data in this buffer to complete the
  // whole clipboard content.
  buffer_ptr m_reply_data;

  // Used to concatenate chunks of data in "m_reply_data" from several
  // PropertyNotify when we are getting the selection owner data with
  // the INCR method.
  size_t m_reply_offset;

  // List of user-defined formats/atoms.
  std::vector<xcb_atom_t> m_custom_formats;
};

Manager* manager = nullptr;

void delete_manager_atexit() {
  if (manager) {
    delete manager;
    manager = nullptr;
  }
}

Manager* get_manager() {
  if (!manager) {
    manager = new Manager;
    std::atexit(delete_manager_atexit);
  }
  return manager;
}

} // anonymous namespace

lock::impl::impl(void*) : m_locked(false) {
  m_locked = get_manager()->try_lock();
}

lock::impl::~impl() {
  if (m_locked)
    manager->unlock();
}

bool lock::impl::clear() {
  manager->clear();
  return true;
}

bool lock::impl::is_convertible(format f) const {
  return manager->is_convertible(f);
}

bool lock::impl::set_data(format f, const char* buf, size_t len) {
  return manager->set_data(f, buf, len);
}

bool lock::impl::get_data(format f, char* buf, size_t len) const {
  return manager->get_data(f, buf, len);
}

size_t lock::impl::get_data_length(format f) const {
  return manager->get_data_length(f);
}

bool lock::impl::set_image(const image& image) {
  return manager->set_image(image);
}

bool lock::impl::get_image(image& output_img) const {
  return manager->get_image(output_img);
}

bool lock::impl::get_image_spec(image_spec& spec) const {
  return manager->get_image_spec(spec);
}

format register_format(const std::string& name) {
  return get_manager()->register_format(name);
}

} // namespace clip

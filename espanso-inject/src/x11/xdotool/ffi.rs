use libc::{c_char, c_int, c_long, useconds_t, wchar_t};

use crate::x11::ffi::{Display, Window};

#[repr(C)]
pub struct charcodemap_t {
  pub key: wchar_t,
  pub code: c_char,
  pub symbol: c_long,
  pub group: c_int,
  pub modmask: c_int,
  pub needs_binding: c_int,
}

#[repr(C)]
pub struct xdo_t {
  pub xdpy: *mut Display,
  pub display_name: *const c_char,
  pub charcodes: *const charcodemap_t,
  pub charcodes_len: c_int,
  pub keycode_high: c_int,
  pub keycode_low: c_int,
  pub keysyms_per_keycode: c_int,
  pub close_display_when_freed: c_int,
  pub quiet: c_int,
  pub debug: c_int,
  pub features_mask: c_int,
}

pub const CURRENTWINDOW: u64 = 0;

#[link(name = "xdotoolvendor", kind = "static")]
extern "C" {
  pub fn xdo_new(display: *const c_char) -> *mut xdo_t;
  pub fn xdo_free(xdo: *const xdo_t);
  pub fn xdo_enter_text_window(
    xdo: *const xdo_t,
    window: Window,
    string: *const c_char,
    delay: useconds_t,
  );
  pub fn fast_send_event(xdo: *const xdo_t, window: Window, keycode: c_int, pressed: c_int);
  pub fn fast_enter_text_window(
    xdo: *const xdo_t,
    window: Window,
    string: *const c_char,
    delay: useconds_t,
  );
}

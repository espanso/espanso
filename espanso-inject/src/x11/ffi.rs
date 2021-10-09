// Some of these structures/methods are taken from the X11-rs project
// https://github.com/erlepereira/x11-rs

use std::{
  ffi::c_void,
  os::raw::{c_char, c_long, c_uint, c_ulong},
};

use libc::c_int;

pub enum Display {}
pub type Window = u64;
pub type Bool = i32;
pub type Time = u64;
pub type KeySym = u64;
pub type KeyCode = u8;

#[allow(non_upper_case_globals)]
pub const KeyPress: c_int = 2;
#[allow(non_upper_case_globals)]
pub const KeyRelease: c_int = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XKeyEvent {
  pub type_: c_int,
  pub serial: c_ulong,
  pub send_event: Bool,
  pub display: *mut Display,
  pub window: Window,
  pub root: Window,
  pub subwindow: Window,
  pub time: Time,
  pub x: c_int,
  pub y: c_int,
  pub x_root: c_int,
  pub y_root: c_int,
  pub state: c_uint,
  pub keycode: c_uint,
  pub same_screen: Bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XModifierKeymap {
  pub max_keypermod: c_int,
  pub modifiermap: *mut KeyCode,
}

#[link(name = "X11")]
extern "C" {
  pub fn XOpenDisplay(name: *const c_char) -> *mut Display;
  pub fn XCloseDisplay(display: *mut Display);
  pub fn XLookupString(
    event: *const XKeyEvent,
    buffer_return: *mut c_char,
    bytes_buffer: c_int,
    keysym_return: *mut KeySym,
    status_in_out: *const c_void,
  ) -> c_int;
  pub fn XDefaultRootWindow(display: *mut Display) -> Window;
  pub fn XGetInputFocus(
    display: *mut Display,
    window_out: *mut Window,
    revert_to: *mut c_int,
  ) -> c_int;
  pub fn XFlush(display: *mut Display) -> c_int;
  pub fn XSendEvent(
    display: *mut Display,
    window: Window,
    propagate: c_int,
    event_mask: c_long,
    event_send: *mut XKeyEvent,
  ) -> c_int;
  pub fn XGetModifierMapping(display: *mut Display) -> *mut XModifierKeymap;
  pub fn XFreeModifiermap(map: *mut XModifierKeymap) -> c_int;
  pub fn XTestFakeKeyEvent(
    display: *mut Display,
    key_code: c_uint,
    is_press: c_int,
    time: c_ulong,
  ) -> c_int;
  pub fn XSync(display: *mut Display, discard: c_int) -> c_int;
  pub fn XQueryKeymap(display: *mut Display, keys_return: *mut u8);
}

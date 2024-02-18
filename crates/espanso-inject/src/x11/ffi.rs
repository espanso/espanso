// Some of these structures/methods are taken from the X11-rs project
// https://github.com/erlepereira/x11-rs

use std::{
  ffi::c_void,
  os::raw::{c_char, c_long, c_uint, c_ulong},
};

use libc::{c_int, c_uchar};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct XModifierKeymap {
  pub max_keypermod: c_int,
  pub modifiermap: *mut KeyCode,
}

// XCreateIC values
#[allow(non_upper_case_globals)]
pub const XIMPreeditNothing: c_int = 0x0008;
#[allow(non_upper_case_globals)]
pub const XIMStatusNothing: c_int = 0x0400;

#[allow(non_upper_case_globals)]
pub const XNClientWindow_0: &[u8] = b"clientWindow\0";
#[allow(non_upper_case_globals)]
pub const XNInputStyle_0: &[u8] = b"inputStyle\0";

pub enum _XIC {}
pub enum _XIM {}
pub enum _XrmHashBucketRec {}

#[allow(clippy::upper_case_acronyms)]
pub type XIC = *mut _XIC;
#[allow(clippy::upper_case_acronyms)]
pub type XIM = *mut _XIM;
pub type XrmDatabase = *mut _XrmHashBucketRec;

#[link(name = "X11")]
extern "C" {
  pub fn XOpenDisplay(name: *const c_char) -> *mut Display;
  pub fn XCloseDisplay(display: *mut Display);
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
  pub fn XOpenIM(
    display: *mut Display,
    db: XrmDatabase,
    res_name: *mut c_char,
    res_class: *mut c_char,
  ) -> XIM;
  pub fn XCreateIC(
    input_method: XIM,
    p2: *const u8,
    p3: c_int,
    p4: *const u8,
    p5: c_int,
    p6: *const c_void,
  ) -> XIC;
  pub fn XDestroyIC(input_context: XIC);
  pub fn XmbResetIC(input_context: XIC) -> *mut c_char;
  pub fn Xutf8LookupString(
    input_context: XIC,
    event: *mut XKeyEvent,
    buffer: *mut c_char,
    buff_size: c_int,
    keysym_return: *mut c_ulong,
    status_return: *mut c_int,
  ) -> c_int;
  pub fn XFilterEvent(event: *mut XKeyEvent, window: c_ulong) -> c_int;
  pub fn XCloseIM(input_method: XIM) -> c_int;
  pub fn XFree(data: *mut c_void) -> c_int;
  pub fn XKeycodeToKeysym(display: *mut Display, keycode: c_uchar, index: c_int) -> c_ulong;
  pub fn XKeysymToString(keysym: c_ulong) -> *mut c_char;
}

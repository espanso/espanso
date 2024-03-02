// Bindings taken from: https://github.com/rtbo/xkbcommon-rs/blob/master/src/xkb/ffi.rs

use std::os::raw::c_int;

use libc::c_char;

#[allow(non_camel_case_types)]
pub enum xkb_context {}
#[allow(non_camel_case_types)]
pub enum xkb_state {}
#[allow(non_camel_case_types)]
pub enum xkb_keymap {}
#[allow(non_camel_case_types)]
pub type xkb_keycode_t = u32;
#[allow(non_camel_case_types)]
pub type xkb_keysym_t = u32;

#[repr(C)]
pub struct xkb_rule_names {
  pub rules: *const c_char,
  pub model: *const c_char,
  pub layout: *const c_char,
  pub variant: *const c_char,
  pub options: *const c_char,
}

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
pub enum xkb_key_direction {
  UP,
  DOWN,
}

#[allow(non_camel_case_types)]
pub type xkb_keymap_compile_flags = u32;
pub const XKB_KEYMAP_COMPILE_NO_FLAGS: u32 = 0;

#[allow(non_camel_case_types)]
pub type xkb_context_flags = u32;
pub const XKB_CONTEXT_NO_FLAGS: u32 = 0;

#[allow(non_camel_case_types)]
pub type xkb_state_component = u32;

pub const EV_KEY: u16 = 0x01;

#[link(name = "xkbcommon")]
extern "C" {
  pub fn xkb_state_unref(state: *mut xkb_state);
  pub fn xkb_state_new(keymap: *mut xkb_keymap) -> *mut xkb_state;
  pub fn xkb_keymap_new_from_names(
    context: *mut xkb_context,
    names: *const xkb_rule_names,
    flags: xkb_keymap_compile_flags,
  ) -> *mut xkb_keymap;
  pub fn xkb_keymap_unref(keymap: *mut xkb_keymap);
  pub fn xkb_context_new(flags: xkb_context_flags) -> *mut xkb_context;
  pub fn xkb_context_unref(context: *mut xkb_context);
  pub fn xkb_state_get_keymap(state: *mut xkb_state) -> *mut xkb_keymap;
  pub fn xkb_keymap_key_repeats(keymap: *mut xkb_keymap, key: xkb_keycode_t) -> c_int;
  pub fn xkb_state_update_key(
    state: *mut xkb_state,
    key: xkb_keycode_t,
    direction: xkb_key_direction,
  ) -> xkb_state_component;
  pub fn xkb_state_key_get_utf8(
    state: *mut xkb_state,
    key: xkb_keycode_t,
    buffer: *mut c_char,
    size: usize,
  ) -> c_int;
  pub fn xkb_state_key_get_one_sym(state: *mut xkb_state, key: xkb_keycode_t) -> xkb_keysym_t;
}

#[link(name = "espansodetectevdev", kind = "static")]
extern "C" {
  pub fn is_keyboard_or_mouse(fd: i32) -> i32;
}

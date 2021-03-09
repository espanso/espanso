// This code is a port of the libxkbcommon "interactive-evdev.c" example
// https://github.com/xkbcommon/libxkbcommon/blob/master/tools/interactive-evdev.c

use std::ffi::CString;

use scopeguard::ScopeGuard;

use anyhow::Result;
use thiserror::Error;

use crate::KeyboardConfig;

use super::{
  context::Context,
  ffi::{
    xkb_keymap, xkb_keymap_new_from_names, xkb_keymap_unref, xkb_rule_names,
    XKB_KEYMAP_COMPILE_NO_FLAGS,
  },
};

pub struct Keymap {
  keymap: *mut xkb_keymap,
}

impl Keymap {
  pub fn new(context: &Context, rmlvo: Option<KeyboardConfig>) -> Result<Keymap> {
    let names = rmlvo.map(|rmlvo| Self::generate_names(rmlvo));

    let names_ptr = names.map_or(std::ptr::null(), |names| &names);
    let raw_keymap = unsafe {
      xkb_keymap_new_from_names(context.get_handle(), names_ptr, XKB_KEYMAP_COMPILE_NO_FLAGS)
    };
    let keymap = scopeguard::guard(raw_keymap, |raw_keymap| unsafe {
      xkb_keymap_unref(raw_keymap);
    });

    if raw_keymap.is_null() {
      return Err(KeymapError::FailedCreation().into());
    }

    Ok(Self {
      keymap: ScopeGuard::into_inner(keymap),
    })
  }

  pub fn get_handle(&self) -> *mut xkb_keymap {
    self.keymap
  }

  fn generate_names(rmlvo: KeyboardConfig) -> xkb_rule_names {
    let rules = rmlvo
      .rules
      .map(|s| CString::new(s).expect("unable to create CString for keymap"));
    let model = rmlvo
      .model
      .map(|s| CString::new(s).expect("unable to create CString for keymap"));
    let layout = rmlvo
      .layout
      .map(|s| CString::new(s).expect("unable to create CString for keymap"));
    let variant = rmlvo
      .variant
      .map(|s| CString::new(s).expect("unable to create CString for keymap"));
    let options = rmlvo
      .options
      .map(|s| CString::new(s).expect("unable to create CString for keymap"));

    xkb_rule_names {
      rules: rules.map_or(std::ptr::null(), |s| s.as_ptr()),
      model: model.map_or(std::ptr::null(), |s| s.as_ptr()),
      layout: layout.map_or(std::ptr::null(), |s| s.as_ptr()),
      variant: variant.map_or(std::ptr::null(), |s| s.as_ptr()),
      options: options.map_or(std::ptr::null(), |s| s.as_ptr()),
    }
  }
}

impl Drop for Keymap {
  fn drop(&mut self) {
    unsafe {
      xkb_keymap_unref(self.keymap);
    }
  }
}

#[derive(Error, Debug)]
pub enum KeymapError {
  #[error("could not create xkb keymap")]
  FailedCreation(),
}

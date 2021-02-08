// This code is a port of the libxkbcommon "interactive-evdev.c" example
// https://github.com/xkbcommon/libxkbcommon/blob/master/tools/interactive-evdev.c

use scopeguard::ScopeGuard;

use anyhow::Result;
use thiserror::Error;

use super::{
  context::Context,
  ffi::{xkb_keymap, xkb_keymap_new_from_names, xkb_keymap_unref, XKB_KEYMAP_COMPILE_NO_FLAGS},
};

pub struct Keymap {
  keymap: *mut xkb_keymap,
}

impl Keymap {
  pub fn new(context: &Context) -> Result<Keymap> {
    let raw_keymap = unsafe {
      xkb_keymap_new_from_names(
        context.get_handle(),
        std::ptr::null(),
        XKB_KEYMAP_COMPILE_NO_FLAGS,
      )
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

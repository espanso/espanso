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
    let owned_rmlvo = Self::generate_owned_rmlvo(rmlvo);
    let names = Self::generate_names(&owned_rmlvo);

    let raw_keymap = unsafe {
      xkb_keymap_new_from_names(context.get_handle(), &names, XKB_KEYMAP_COMPILE_NO_FLAGS)
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

  fn generate_owned_rmlvo(rmlvo: Option<KeyboardConfig>) -> OwnedRawKeyboardConfig {
    let rules = rmlvo
      .as_ref()
      .and_then(|config| config.rules.clone())
      .unwrap_or_default();
    let model = rmlvo
      .as_ref()
      .and_then(|config| config.model.clone())
      .unwrap_or_default();
    let layout = rmlvo
      .as_ref()
      .and_then(|config| config.layout.clone())
      .unwrap_or_default();
    let variant = rmlvo
      .as_ref()
      .and_then(|config| config.variant.clone())
      .unwrap_or_default();
    let options = rmlvo
      .as_ref()
      .and_then(|config| config.options.clone())
      .unwrap_or_default();

    OwnedRawKeyboardConfig {
      rules: CString::new(rules).expect("unable to create CString for keymap"),
      model: CString::new(model).expect("unable to create CString for keymap"),
      layout: CString::new(layout).expect("unable to create CString for keymap"),
      variant: CString::new(variant).expect("unable to create CString for keymap"),
      options: CString::new(options).expect("unable to create CString for keymap"),
    }
  }

  fn generate_names(owned_config: &OwnedRawKeyboardConfig) -> xkb_rule_names {
    xkb_rule_names {
      rules: owned_config.rules.as_ptr(),
      model: owned_config.model.as_ptr(),
      layout: owned_config.layout.as_ptr(),
      variant: owned_config.variant.as_ptr(),
      options: owned_config.options.as_ptr(),
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

struct OwnedRawKeyboardConfig {
  rules: CString,
  model: CString,
  layout: CString,
  variant: CString,
  options: CString,
}

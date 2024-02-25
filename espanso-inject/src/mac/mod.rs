/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

mod raw_keys;

use std::{ffi::CString, os::raw::c_char};

use log::error;
use raw_keys::convert_key_to_vkey;

use anyhow::Result;
use thiserror::Error;

use crate::{keys, InjectionOptions, Injector};

#[allow(improper_ctypes)]
#[link(name = "espansoinject", kind = "static")]
extern "C" {
  pub fn inject_string(string: *const c_char, delay: i32);
  pub fn inject_separate_vkeys(vkey_array: *const i32, vkey_count: i32, delay: i32);
  pub fn inject_vkeys_combination(vkey_array: *const i32, vkey_count: i32, delay: i32);
}

pub struct MacInjector {}

#[allow(clippy::new_without_default)]
impl MacInjector {
  pub fn new() -> Self {
    Self {}
  }

  pub fn convert_to_vk_array(keys: &[keys::Key]) -> Result<Vec<i32>> {
    let mut virtual_keys: Vec<i32> = Vec::new();
    for key in keys {
      let vk = convert_key_to_vkey(key);
      if let Some(vk) = vk {
        virtual_keys.push(vk);
      } else {
        return Err(MacInjectorError::MappingFailure(key.clone()).into());
      }
    }
    Ok(virtual_keys)
  }
}

impl Injector for MacInjector {
  fn send_string(&self, string: &str, options: InjectionOptions) -> Result<()> {
    let c_string = CString::new(string)?;
    unsafe {
      inject_string(c_string.as_ptr(), options.delay);
    }
    Ok(())
  }

  fn send_keys(&self, keys: &[keys::Key], options: InjectionOptions) -> Result<()> {
    let virtual_keys = Self::convert_to_vk_array(keys)?;

    unsafe {
      inject_separate_vkeys(
        virtual_keys.as_ptr(),
        virtual_keys.len() as i32,
        options.delay,
      );
    }

    Ok(())
  }

  fn send_key_combination(&self, keys: &[keys::Key], options: InjectionOptions) -> Result<()> {
    let virtual_keys = Self::convert_to_vk_array(keys)?;

    unsafe {
      inject_vkeys_combination(
        virtual_keys.as_ptr(),
        virtual_keys.len() as i32,
        options.delay,
      );
    }

    Ok(())
  }
}

#[derive(Error, Debug)]
pub enum MacInjectorError {
  #[error("missing vkey mapping for key `{0}`")]
  MappingFailure(keys::Key),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn convert_raw_to_virtual_key_array() {
    assert_eq!(
      MacInjector::convert_to_vk_array(&[keys::Key::Alt, keys::Key::V]).unwrap(),
      vec![0x3A, 0x09]
    );
  }
}

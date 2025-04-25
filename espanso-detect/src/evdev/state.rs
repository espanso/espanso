// This code is a port of the libxkbcommon "interactive-evdev.c" example
// https://github.com/xkbcommon/libxkbcommon/blob/master/tools/interactive-evdev.c

use scopeguard::ScopeGuard;

use anyhow::Result;
use thiserror::Error;

use super::{
    ffi::{xkb_state, xkb_state_key_get_one_sym, xkb_state_new, xkb_state_unref},
    keymap::Keymap,
};

pub struct State {
    state: *mut xkb_state,
}

impl State {
    pub fn new(keymap: &Keymap) -> Result<State> {
        let raw_state = unsafe { xkb_state_new(keymap.get_handle()) };
        let state = scopeguard::guard(raw_state, |raw_state| unsafe {
            xkb_state_unref(raw_state);
        });

        if raw_state.is_null() {
            return Err(StateError::FailedCreation().into());
        }

        Ok(Self {
            state: ScopeGuard::into_inner(state),
        })
    }

    pub fn get_sym(&self, code: u32) -> Option<u32> {
        let sym = unsafe { xkb_state_key_get_one_sym(self.state, code) };
        if sym == 0 {
            None
        } else {
            Some(sym)
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            xkb_state_unref(self.state);
        }
    }
}

#[derive(Error, Debug)]
pub enum StateError {
    #[error("could not create xkb state")]
    FailedCreation(),
}

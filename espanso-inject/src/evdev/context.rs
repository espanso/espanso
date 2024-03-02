// This code is a port of the libxkbcommon "interactive-evdev.c" example
// https://github.com/xkbcommon/libxkbcommon/blob/master/tools/interactive-evdev.c

use scopeguard::ScopeGuard;

use super::ffi::{xkb_context, xkb_context_new, xkb_context_unref, XKB_CONTEXT_NO_FLAGS};
use anyhow::Result;
use thiserror::Error;

pub struct Context {
    context: *mut xkb_context,
}

impl Context {
    pub fn new() -> Result<Context> {
        let raw_context = unsafe { xkb_context_new(XKB_CONTEXT_NO_FLAGS) };
        let context = scopeguard::guard(raw_context, |raw_context| unsafe {
            xkb_context_unref(raw_context);
        });

        if raw_context.is_null() {
            return Err(ContextError::FailedCreation().into());
        }

        Ok(Self {
            context: ScopeGuard::into_inner(context),
        })
    }

    pub fn get_handle(&self) -> *mut xkb_context {
        self.context
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            xkb_context_unref(self.context);
        }
    }
}

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("could not create xkb context")]
    FailedCreation(),
}

/*
 * This file is part of modulo.
 *
 * Copyright (C) 2020-2021 Federico Terzi
 *
 * modulo is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * modulo is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with modulo.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{LazyLock, Mutex};

use anyhow::Result;

use crate::match_explain_dialog::{MatchExplainDialogHandlers, MatchExplainDialogOptions};
use crate::sys::interop::MatchExplainDialogMetadata;
use crate::sys::util::convert_to_cstring_or_null;

static HANDLERS: LazyLock<Mutex<Option<MatchExplainDialogHandlers>>> =
    LazyLock::new(|| Mutex::new(None));
static LAST_OUTPUT: LazyLock<Mutex<Option<CString>>> = LazyLock::new(|| Mutex::new(None));

pub fn show(options: MatchExplainDialogOptions) -> Result<()> {
    let (_c_window_icon_path, c_window_icon_path_ptr) =
        convert_to_cstring_or_null(options.window_icon_path);

    extern "C" fn on_check(
        trigger: *const c_char,
        show_all: c_int,
        json_output: c_int,
    ) -> *const c_char {
        let trigger_str = if trigger.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(trigger) }
                .to_string_lossy()
                .to_string()
        };

        let show_all = show_all != 0;
        let json_output = json_output != 0;

        let lock = HANDLERS
            .lock()
            .expect("unable to acquire handlers lock in on_check");
        let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");

        let output = (handlers_ref.on_check)(&trigger_str, show_all, json_output);
        let sanitized = output.replace('\0', "\\0");
        let c_output = CString::new(sanitized).unwrap_or_else(|_| CString::new("").unwrap());

        let mut output_lock = LAST_OUTPUT
            .lock()
            .expect("unable to acquire output lock in on_check");
        *output_lock = Some(c_output);

        output_lock
            .as_ref()
            .expect("unable to read output CString")
            .as_ptr()
    }

    extern "C" fn on_focus_gained() {
        let lock = HANDLERS
            .lock()
            .expect("unable to acquire handlers lock in on_focus_gained");
        let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
        (handlers_ref.on_focus_gained)();
    }

    extern "C" fn on_focus_lost() {
        let lock = HANDLERS
            .lock()
            .expect("unable to acquire handlers lock in on_focus_lost");
        let handlers_ref = (*lock).as_ref().expect("unable to unwrap handlers");
        (handlers_ref.on_focus_lost)();
    }

    {
        let mut lock = HANDLERS.lock().expect("unable to acquire handlers lock");
        *lock = Some(options.handlers);
    }

    let metadata = MatchExplainDialogMetadata {
        window_icon_path: c_window_icon_path_ptr,
        on_check,
        on_focus_gained,
        on_focus_lost,
    };

    unsafe {
        super::interop::interop_show_match_explain_dialog(&metadata);
    }

    Ok(())
}

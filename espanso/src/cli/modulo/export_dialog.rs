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

use crate::icon::IconPaths;
use clap::ArgMatches;
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::Mutex;

// Global state to pass paths to the C callback
static EXPORT_CONTEXT: Mutex<Option<ExportContext>> = Mutex::new(None);

struct ExportContext {
    runtime_path: PathBuf,
    config_path: PathBuf,
}

extern "C" fn generate_export_code_callback(
    export_config: std::os::raw::c_int,
    export_matches: std::os::raw::c_int,
    export_packages: std::os::raw::c_int,
) -> *const c_char {
    let context = EXPORT_CONTEXT.lock().unwrap();
    if let Some(ctx) = context.as_ref() {
        match crate::cli::offline::export_config_to_string(
            &ctx.runtime_path,
            &ctx.config_path,
            export_config != 0,
            export_matches != 0,
            export_packages != 0,
        ) {
            Ok(export_code) => {
                let c_string = CString::new(export_code).unwrap_or_else(|_| CString::new("Error: Invalid export code").unwrap());
                c_string.into_raw()
            }
            Err(err) => {
                let error_msg = format!("Error: {}", err);
                let c_string = CString::new(error_msg).unwrap_or_else(|_| CString::new("Error generating export code").unwrap());
                c_string.into_raw()
            }
        }
    } else {
        let c_string = CString::new("Error: Export context not initialized").unwrap();
        c_string.into_raw()
    }
}

pub fn export_dialog_main(args: &ArgMatches, icon_paths: &IconPaths) -> i32 {
    let runtime_path = args.value_of("runtime_path").expect("missing runtime_path");
    let config_path = args.value_of("config_path").expect("missing config_path");

    // Set up the export context for the callback
    {
        let mut context = EXPORT_CONTEXT.lock().unwrap();
        *context = Some(ExportContext {
            runtime_path: PathBuf::from(runtime_path),
            config_path: PathBuf::from(config_path),
        });
    }

    // Show the custom export dialog with checkboxes and live preview
    espanso_modulo::export_dialog::show(
        icon_paths.wizard_icon.as_ref().map(|p| p.to_string_lossy().to_string()).as_deref(),
        generate_export_code_callback
    );

    // Clean up the context
    {
        let mut context = EXPORT_CONTEXT.lock().unwrap();
        *context = None;
    }

    0
}

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

use crate::gui::modulo::manager::ModuloManager;
use crate::gui::FormUI;
use crate::path::Paths;
use espanso_engine::dispatch::ExportImportHandler;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Mutex;

// Global state to pass paths to the C callback
static EXPORT_CONTEXT: Mutex<Option<ExportContext>> = Mutex::new(None);

struct ExportContext {
    runtime_path: String,
    config_path: String,
}

pub struct ExportImportHandlerAdapter<'a> {
    modulo_manager: &'a ModuloManager,
    form_ui: &'a dyn FormUI,
    paths: &'a Paths,
}

impl<'a> ExportImportHandlerAdapter<'a> {
    pub fn new(modulo_manager: &'a ModuloManager, form_ui: &'a dyn FormUI, paths: &'a Paths) -> Self {
        Self {
            modulo_manager,
            form_ui,
            paths,
        }
    }
}

extern "C" fn generate_export_code_callback(
    export_config: std::os::raw::c_int,
    export_matches: std::os::raw::c_int,
    export_packages: std::os::raw::c_int,
) -> *const c_char {
    let context = EXPORT_CONTEXT.lock().unwrap();
    if let Some(ctx) = context.as_ref() {
        match crate::cli::offline::export_config_to_string(
            &std::path::PathBuf::from(&ctx.runtime_path),
            &std::path::PathBuf::from(&ctx.config_path),
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

impl ExportImportHandler for ExportImportHandlerAdapter<'_> {
    fn handle_export(&self) -> anyhow::Result<()> {
        let runtime_path = self.paths.runtime.to_string_lossy().to_string();
        let config_path = self.paths.config.to_string_lossy().to_string();
        
        // Show the export dialog with paths passed as arguments (spawned as separate process)
        self.modulo_manager.spawn(
            &["export_dialog", "--runtime-path", &runtime_path, "--config-path", &config_path],
            ""
        )?;

        Ok(())
    }

    fn handle_import(&self) -> anyhow::Result<()> {
        let config_path = self.paths.config.to_string_lossy().to_string();

        // Show the import dialog (spawned as separate process)
        self.modulo_manager.spawn(
            &["import_dialog", "--config-path", &config_path],
            ""
        )?;

        Ok(())
    }
}
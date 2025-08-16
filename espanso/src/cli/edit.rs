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

use std::path::{Path, PathBuf};
use std::process::Command;

use super::{CliModule, CliModuleArgs};

#[cfg(target_os = "linux")]
fn default_editor() -> String {
    "/bin/nano".to_owned()
}
#[cfg(target_os = "macos")]
fn default_editor() -> String {
    "/usr/bin/nano".to_owned()
}
#[cfg(target_os = "windows")]
fn default_editor() -> String {
    "C:\\Windows\\System32\\notepad.exe".to_owned()
}

pub fn new() -> CliModule {
    CliModule {
        requires_paths: true,
        subcommand: "edit".to_string(),
        entry: edit_main,
        ..Default::default()
    }
}

fn edit_main(args: CliModuleArgs) -> i32 {
    let paths = args.paths.expect("missing paths argument");
    let cli_args = args.cli_args.expect("missing cli_args");

    assert!(
        paths.config.is_dir(),
        "config directory does not exist in path: {}",
        paths.config.display()
    );

    // Determine which is the file to edit
    let target_file = cli_args.value_of("target_file");
    let target_path = determine_target_path(&paths.config, target_file);

    println!(
        "Editing file: {}",
        &target_path.to_string_lossy().to_string()
    );

    open_editor(&target_path);

    0
}

fn determine_target_path(config_path: &Path, target_file: Option<&str>) -> PathBuf {
    if let Some(target_file) = target_file {
        match target_file {
            "default" => config_path.join("config").join("default.yml"),
            "base" => config_path.join("match").join("base.yml"),
            custom => {
                if !std::path::Path::new(custom)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("yml"))
                    && !std::path::Path::new(custom)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("yaml"))
                {
                    config_path.join("match").join(format!("{custom}.yml"))
                } else {
                    config_path.join(custom)
                }
            }
        }
    } else {
        config_path.join("match").join("base.yml")
    }
}

pub fn open_editor(file_path: &Path) -> bool {
    // Check if another editor is defined in the environment variables
    let editor_var = std::env::var_os("EDITOR");
    let visual_var = std::env::var_os("VISUAL");

    // Prioritize the editors specified by the environment variable, use the default one
    let editor: String = if let Some(editor_var) = editor_var {
        editor_var.to_string_lossy().to_string()
    } else if let Some(visual_var) = visual_var {
        visual_var.to_string_lossy().to_string()
    } else {
        default_editor()
    };

    // Start the editor and wait for its termination
    let status = if cfg!(target_os = "windows") {
        Command::new(&editor).arg(file_path).spawn()
    } else {
        // On Unix, spawn the editor using the shell so that it can
        // accept parameters. See issue #245
        Command::new("/bin/bash")
            .arg("-c")
            .arg(format!("{} '{}'", editor, file_path.to_string_lossy()))
            .spawn()
    };

    if let Ok(mut child) = status {
        // Wait for the user to edit the configuration
        let result = child.wait();

        if let Ok(exit_status) = result {
            exit_status.success()
        } else {
            false
        }
    } else {
        println!("Error: could not start editor at: {}", &editor);
        false
    }
}

// fn restart_espanso(paths_overrides: &PathsOverrides) -> Result<()> {
//   let espanso_exe_path = std::env::current_exe()?;

//   let mut command = Command::new(&espanso_exe_path.to_string_lossy().to_string());
//   command.args(&["restart"]);
//   command.with_paths_overrides(paths_overrides);

//   let output = command.output()?;

//   if output.status.success() {
//     Ok(())
//   } else {
//     bail!("restart command returned a non-zero exit code");
//   }
// }

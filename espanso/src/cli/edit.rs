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
    "config directory does not exist in path: {:?}",
    paths.config
  );

  // Determine which is the file to edit
  let target_file = cli_args.value_of("target_file");
  let target_path = determine_target_path(&paths.config, target_file);

  println!(
    "Editing file: {}",
    &target_path.to_string_lossy().to_string()
  );

  open_editor(&target_path);

  // The previous version automatically reloaded the config after saving.
  // Given that the new version automatically reloads config changes, this could be avoided.
  // Nevertheless, this assumption might be wrong, so I'm keeping the necessary code.
  // TODO: evaluate if reload is needed after v2 becomes stable

  // // Based on the fact that the file already exists or not, we should detect in different
  // // ways if a reload is needed
  // let should_reload = if target_path.exists() {
  //   // Get the last modified date, so that we can detect if the user actually edits the file
  //   // before reloading
  //   let metadata = std::fs::metadata(&target_path).expect("cannot gather file metadata");
  //   let last_modified = metadata
  //     .modified()
  //     .expect("cannot read file last modified date");

  //   let result = open_editor(&target_path);
  //   if result {
  //     let new_metadata = std::fs::metadata(&target_path).expect("cannot gather file metadata");
  //     let new_last_modified = new_metadata
  //       .modified()
  //       .expect("cannot read file last modified date");

  //     if last_modified != new_last_modified {
  //       println!("File has been modified, reloading configuration");
  //       true
  //     } else {
  //       println!("File has not been modified, avoiding reload");
  //       false
  //     }
  //   } else {
  //     false
  //   }
  // } else {
  //   let result = open_editor(&target_path);
  //   if result {
  //     // If the file has been created, we should reload the espanso config
  //     if target_path.exists() {
  //       println!("A new file has been created, reloading configuration");
  //       true
  //     } else {
  //       println!("No file has been created, avoiding reload");
  //       false
  //     }
  //   } else {
  //     false
  //   }
  // };

  // let no_restart: bool = if cli_args.is_present("norestart") {
  //   println!("Skipping automatic restart");
  //   true
  // } else {
  //   false
  // };

  // if should_reload && !no_restart {
  //   // Check if the new configuration is valid

  //   if let Err(err) = crate::config::load_config(&paths.config, &paths.packages) {
  //     eprintln!("Unable to reload espanso due to a configuration error:");
  //     eprintln!("{:?}", err);
  //     return 1;
  //   };

  //   restart_espanso(&paths_overrides).expect("unable to restart espanso");
  // }

  0
}

fn determine_target_path(config_path: &Path, target_file: Option<&str>) -> PathBuf {
  if let Some(target_file) = target_file {
    match target_file {
      "default" => {
        if espanso_config::is_legacy_config(config_path) {
          config_path.join("default.yml")
        } else {
          config_path.join("config").join("default.yml")
        }
      }
      "base" => {
        if espanso_config::is_legacy_config(config_path) {
          panic!("'base' alias cannot be used in compatibility mode, please migrate your configuration by running 'espanso migrate'")
        } else {
          config_path.join("match").join("base.yml")
        }
      }
      custom => {
        if !std::path::Path::new(custom)
          .extension()
          .map_or(false, |ext| ext.eq_ignore_ascii_case("yml"))
          && !std::path::Path::new(custom)
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("yaml"))
        {
          if espanso_config::is_legacy_config(config_path) {
            config_path.join("user").join(format!("{custom}.yml"))
          } else {
            config_path.join("match").join(format!("{custom}.yml"))
          }
        } else {
          config_path.join(custom)
        }
      }
    }
  } else if espanso_config::is_legacy_config(config_path) {
    config_path.join("default.yml")
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

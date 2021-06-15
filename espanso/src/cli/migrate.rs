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

use std::{path::PathBuf, sync::Mutex};

use crate::{exit_code::{MIGRATE_ALREADY_NEW_FORMAT, MIGRATE_CLEAN_FAILURE, MIGRATE_DIRTY_FAILURE, MIGRATE_LEGACY_INSTANCE_RUNNING, MIGRATE_SUCCESS, MIGRATE_UNEXPECTED_FAILURE, MIGRATE_USER_ABORTED}, lock::acquire_legacy_lock};

use super::{CliModule, CliModuleArgs};
use colored::*;
use dialoguer::Confirm;
use fs_extra::dir::CopyOptions;
use tempdir::TempDir;

lazy_static! {
  static ref CURRENT_PANIC_EXIT_CODE: Mutex<i32> = Mutex::new(MIGRATE_UNEXPECTED_FAILURE);
}

pub fn new() -> CliModule {
  CliModule {
    requires_paths: true,
    requires_config: true,
    subcommand: "migrate".to_string(),
    entry: migrate_main,
    ..Default::default()
  }
}

fn migrate_main(args: CliModuleArgs) -> i32 {
  let paths = args.paths.expect("missing paths argument");
  let cli_args = args.cli_args.expect("missing cli_args");

  configure_custom_panic_hook();

  if !args.is_legacy_config {
    eprintln!("Can't migrate configurations, as the default directory [1] is already encoded with the new format");
    eprintln!("[1]: {:?}", paths.config);
    eprintln!("The migration tool is only meant to convert the espanso's legacy configuration format (prior to");
    eprintln!("version 0.7.3) to the new one (since version 2.0)");
    return MIGRATE_ALREADY_NEW_FORMAT;
  }

  let legacy_lock_file = acquire_legacy_lock(&paths.runtime);
  if legacy_lock_file.is_none() {
    eprintln!("An instance of legacy espanso is running, please terminate it, otherwise the migration can't be performed");
    return MIGRATE_LEGACY_INSTANCE_RUNNING;
  }

  let target_backup_dir = find_available_backup_dir();

  println!("\n{}\n", "Welcome to espanso v2!".bold());
  println!("This migration tool will help you to smoothly transition to the new espanso v2 configuration format.");
  println!("");
  println!(
    "1. Firstly, espanso will {} your current configuration, located in:\n",
    "backup".green().bold()
  );
  println!("   {}\n", paths.config.to_string_lossy().italic());
  println!("   into this folder:\n");
  println!("   {}\n", target_backup_dir.to_string_lossy().italic());
  println!(
    "2. Then, it will {} your configuration to the new format, replacing",
    "convert".bold().green()
  );
  println!("   the current content of the config directory.");
  println!("");

  if !cli_args.is_present("noconfirm") {
    if !Confirm::new()
      .with_prompt("Do you want to proceed?")
      .default(true)
      .interact()
      .expect("unable to read choice")
    {
      return MIGRATE_USER_ABORTED;
    }
  }

  println!("Backing up your configuration...");
  update_panic_exit_code(MIGRATE_CLEAN_FAILURE);

  fs_extra::dir::copy(
    &paths.config,
    &target_backup_dir,
    &CopyOptions {
      copy_inside: true,
      ..Default::default()
    },
  )
  .expect("unable to backup the current config");
  println!("{}", "Backup completed!".green());

  println!("Converting the configuration...");
  let temp_dir = TempDir::new("espanso-migrate-out").expect("unable to create temporary directory");
  let temp_out_dir = temp_dir.path().join("out");
  espanso_migrate::migrate(&paths.config, &paths.packages, &temp_out_dir)
    .expect("an error occurred while converting the configuration");
  println!("{}", "Conversion completed!".green());

  println!("Replacing old configuration with the new one...");
  update_panic_exit_code(MIGRATE_DIRTY_FAILURE);

  let mut to_be_removed = Vec::new();
  let legacy_dir_content =
    fs_extra::dir::get_dir_content(&paths.config).expect("unable to list legacy dir files");
  to_be_removed.extend(legacy_dir_content.files);
  to_be_removed.extend(legacy_dir_content.directories);
  fs_extra::remove_items(&to_be_removed).expect("unable to remove previous configuration");
  fs_extra::dir::copy(
    &temp_out_dir,
    &paths.config,
    &CopyOptions {
      copy_inside: true,
      ..Default::default()
    },
  )
  .expect("unable to copy new configuration into target location");

  let target_packages_dir = &paths.config.join("match").join("packages");
  if !target_packages_dir.is_dir() {
    std::fs::create_dir_all(target_packages_dir).expect("unable to create new packages directory");
  }

  println!("{}", "Configuration successfully migrated!".green());

  MIGRATE_SUCCESS
}

fn find_available_backup_dir() -> PathBuf {
  for i in 1..20 {
    let num = if i > 1 {
      format!("-{}", i)
    } else {
      "".to_string()
    };

    let target_backup_dir = dirs::document_dir()
      .expect("unable to generate backup directory")
      .join(format!("espanso-migrate-backup{}", num));

    if !target_backup_dir.is_dir() {
      return target_backup_dir;
    }
  }

  panic!("could not generate valid backup directory");
}

fn configure_custom_panic_hook() {
  let previous_hook = std::panic::take_hook();
  std::panic::set_hook(Box::new(move |info| {
    (*previous_hook)(info);

    let exit_code = CURRENT_PANIC_EXIT_CODE.lock().unwrap();
    std::process::exit(*exit_code);
  }));
}

fn update_panic_exit_code(exit_code: i32) {
  let mut lock = CURRENT_PANIC_EXIT_CODE.lock().expect("unable to update panic exit code");
  *lock = exit_code;
}
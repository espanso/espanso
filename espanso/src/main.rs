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

// This is needed to avoid showing a console window when starting espanso on Windows
// TODO #![windows_subsystem = "windows"]

#[macro_use]
extern crate lazy_static;

use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use cli::{CliModule, CliModuleArgs};
use log::{error, info};
use logging::FileProxy;
use simplelog::{
  CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};

use crate::cli::LogMode;

mod cli;
mod engine;
mod gui;
mod ipc;
mod lock;
mod logging;
mod util;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOG_FILE_NAME: &str = "espanso.log";

lazy_static! {
  static ref CLI_HANDLERS: Vec<CliModule> = vec![
    cli::path::new(),
    cli::log::new(),
    cli::worker::new(),
    cli::daemon::new()
  ];
}

fn main() {
  // TODO: attach console

  let install_subcommand = SubCommand::with_name("install")
    .about("Install a package. Equivalent to 'espanso package install'")
    .arg(
      Arg::with_name("external")
        .short("e")
        .long("external")
        .required(false)
        .takes_value(false)
        .help("Allow installing packages from non-verified repositories."),
    )
    .arg(Arg::with_name("package_name").help("Package name"))
    .arg(
      Arg::with_name("repository_url")
        .help("(Optional) Link to GitHub repository")
        .required(false)
        .default_value("hub"),
    )
    .arg(
      Arg::with_name("proxy")
        .help("Use a proxy, should be used as --proxy=https://proxy:1234")
        .required(false)
        .long("proxy")
        .takes_value(true),
    );

  let uninstall_subcommand = SubCommand::with_name("uninstall")
    .about("Remove an installed package. Equivalent to 'espanso package uninstall'")
    .arg(Arg::with_name("package_name").help("Package name"));

  let mut clap_instance = App::new("espanso")
    .version(VERSION)
    .author("Federico Terzi")
    .about("A Privacy-first, Cross-platform Text Expander")
    .arg(
      Arg::with_name("v")
        .short("v")
        .multiple(true)
        .help("Sets the level of verbosity"),
    )
    .arg(
      Arg::with_name("config_dir")
        .long("config_dir")
        .takes_value(true)
        .help("Specify a custom path from which espanso should read the configuration"),
    )
    .arg(
      Arg::with_name("package_dir")
        .long("package_dir")
        .takes_value(true)
        .help("Specify a custom path for the espanso package directory"),
    )
    .arg(
      Arg::with_name("runtime_dir")
        .long("runtime_dir")
        .takes_value(true)
        .help("Specify a custom path for the espanso runtime directory"),
    )
    // .subcommand(SubCommand::with_name("cmd")
    //     .about("Send a command to the espanso daemon.")
    //     .subcommand(SubCommand::with_name("exit")
    //         .about("Terminate the daemon."))
    //     .subcommand(SubCommand::with_name("enable")
    //         .about("Enable the espanso replacement engine."))
    //     .subcommand(SubCommand::with_name("disable")
    //         .about("Disable the espanso replacement engine."))
    //     .subcommand(SubCommand::with_name("toggle")
    //         .about("Toggle the status of the espanso replacement engine."))
    // )
    // .subcommand(SubCommand::with_name("edit")
    //     .about("Open the default text editor to edit config files and reload them automatically when exiting")
    //     .arg(Arg::with_name("config")
    //         .help("Defaults to \"default\". The configuration file name to edit (without the .yml extension)."))
    //     .arg(Arg::with_name("norestart")
    //         .short("n")
    //         .long("norestart")
    //         .required(false)
    //         .takes_value(false)
    //         .help("Avoid restarting espanso after editing the file"))
    // )
    // .subcommand(SubCommand::with_name("detect")
    //     .about("Tool to detect current window properties, to simplify filters creation."))
    .subcommand(SubCommand::with_name("daemon")
      .setting(AppSettings::Hidden)
      .about("Start the daemon without spawning a new process."))
    // .subcommand(SubCommand::with_name("register")
    //     .about("MacOS and Linux only. Register espanso in the system daemon manager."))
    // .subcommand(SubCommand::with_name("unregister")
    //     .about("MacOS and Linux only. Unregister espanso from the system daemon manager."))
    .subcommand(SubCommand::with_name("log").about("Print the daemon logs."))
    // .subcommand(SubCommand::with_name("start")
    //     .about("Start the daemon spawning a new process in the background."))
    // .subcommand(SubCommand::with_name("stop")
    //     .about("Stop the espanso daemon."))
    // .subcommand(SubCommand::with_name("restart")
    //     .about("Restart the espanso daemon."))
    // .subcommand(SubCommand::with_name("status")
    //     .about("Check if the espanso daemon is running or not."))
    .subcommand(
      SubCommand::with_name("path")
        .about("Prints all the espanso directory paths to easily locate configuration and matches.")
        .subcommand(SubCommand::with_name("config").about("Print the current config folder path."))
        .subcommand(
          SubCommand::with_name("packages").about("Print the current packages folder path."),
        )
        .subcommand(
          SubCommand::with_name("data")
            .about("Print the current data folder path.")
            .setting(AppSettings::Hidden),
        ) // Legacy path
        .subcommand(
          SubCommand::with_name("runtime").about("Print the current runtime folder path."),
        )
        .subcommand(
          SubCommand::with_name("default").about("Print the default configuration file path."),
        )
        .subcommand(SubCommand::with_name("base").about("Print the default match file path.")),
    )
    // .subcommand(SubCommand::with_name("match")
    //     .about("List and execute matches from the CLI")
    //     .subcommand(SubCommand::with_name("list")
    //         .about("Print all matches to standard output")
    //         .arg(Arg::with_name("json")
    //             .short("j")
    //             .long("json")
    //             .help("Return the matches as json")
    //             .required(false)
    //             .takes_value(false)
    //         )
    //         .arg(Arg::with_name("onlytriggers")
    //             .short("t")
    //             .long("onlytriggers")
    //             .help("Print only triggers without replacement")
    //             .required(false)
    //             .takes_value(false)
    //         )
    //         .arg(Arg::with_name("preservenewlines")
    //             .short("n")
    //             .long("preservenewlines")
    //             .help("Preserve newlines when printing replacements")
    //             .required(false)
    //             .takes_value(false)
    //         )
    //     )
    //     .subcommand(SubCommand::with_name("exec")
    //         .about("Triggers the expansion of the given match")
    //         .arg(Arg::with_name("trigger")
    //             .help("The trigger of the match to be expanded")
    //         )
    //     )
    // )
    // Package manager
    // .subcommand(SubCommand::with_name("package")
    //     .about("Espanso package manager commands")
    //     .subcommand(install_subcommand.clone())
    //     .subcommand(uninstall_subcommand.clone())
    //     .subcommand(SubCommand::with_name("list")
    //         .about("List all installed packages")
    //         .arg(Arg::with_name("full")
    //             .help("Print all package info")
    //             .long("full")))
    //     .subcommand(SubCommand::with_name("refresh")
    //         .about("Update espanso package index"))
    // )
    .subcommand(
      SubCommand::with_name("worker")
        .setting(AppSettings::Hidden)
        .arg(
          Arg::with_name("reload")
            .short("r")
            .long("reload")
            .required(false)
            .takes_value(false),
        ),
    );
  // .subcommand(install_subcommand)
  // .subcommand(uninstall_subcommand);

  // TODO: explain that the register and unregister commands are only meaningful
  // when using the system daemon manager on macOS and Linux

  let matches = clap_instance.clone().get_matches();
  let log_level = match matches.occurrences_of("v") {
    0 | 1 => LevelFilter::Info,

    // Trace mode is only available in debug mode for security reasons
    #[cfg(debug_assertions)]
    3 => LevelFilter::Trace,

    _ => LevelFilter::Debug,
  };

  let handler = CLI_HANDLERS
    .iter()
    .find(|cli| matches.subcommand_matches(&cli.subcommand).is_some());

  if let Some(handler) = handler {
    let log_proxy = FileProxy::new();
    if handler.enable_logs {
      let config = ConfigBuilder::new()
        .set_time_to_local(true)
        .set_time_format(format!("%H:%M:%S [{}({})]", handler.subcommand, std::process::id()))
        .set_location_level(LevelFilter::Off)
        .add_filter_ignore_str("html5ever")
        .build();

      CombinedLogger::init(vec![
        TermLogger::new(log_level, config.clone(), TerminalMode::Mixed),
        WriteLogger::new(LevelFilter::Info, config, log_proxy.clone()),
      ])
      .expect("unable to initialize logs");

      // Activate logging for panics
      log_panics::init();
    }

    let mut cli_args: CliModuleArgs = CliModuleArgs::default();

    if handler.requires_paths || handler.requires_config {
      let force_config_path = get_path_override(&matches, "config_dir", "ESPANSO_CONFIG_DIR");
      let force_package_path = get_path_override(&matches, "package_dir", "ESPANSO_PACKAGE_DIR");
      let force_runtime_path = get_path_override(&matches, "runtime_dir", "ESPANSO_RUNTIME_DIR");

      let paths = espanso_path::resolve_paths(
        force_config_path.as_deref(),
        force_package_path.as_deref(),
        force_runtime_path.as_deref(),
      );

      info!("reading configs from: {:?}", paths.config);
      info!("reading packages from: {:?}", paths.packages);
      info!("using runtime dir: {:?}", paths.runtime);

      if handler.requires_config {
        let (config_store, match_store, is_legacy_config) =
          if espanso_config::is_legacy_config(&paths.config) {
            let (config_store, match_store) =
              espanso_config::load_legacy(&paths.config, &paths.packages)
                .expect("unable to load legacy config");
            (config_store, match_store, true)
          } else {
            let (config_store, match_store) =
              espanso_config::load(&paths.config).expect("unable to load config");
            (config_store, match_store, false)
          };

        cli_args.is_legacy_config = is_legacy_config;
        cli_args.config_store = Some(config_store);
        cli_args.match_store = Some(match_store);
      }

      if handler.enable_logs {
        log_proxy
          .set_output_file(
            &paths.runtime.join(LOG_FILE_NAME),
            handler.log_mode == LogMode::Read,
            handler.log_mode == LogMode::CleanAndAppend,
          )
          .expect("unable to set up log output file");
      }

      cli_args.paths = Some(paths);
    }

    if let Some(args) = matches.subcommand_matches(&handler.subcommand) {
      cli_args.cli_args = Some(args.clone());
    }

    (handler.entry)(cli_args)
  } else {
    clap_instance
      .print_long_help()
      .expect("unable to print help");
    println!();
  }
}

fn get_path_override(matches: &ArgMatches, argument: &str, env_var: &str) -> Option<PathBuf> {
  if let Some(path) = matches.value_of(argument) {
    let path = PathBuf::from(path.trim());
    if path.is_dir() {
      return Some(path);
    } else {
      error!("{} argument was specified, but it doesn't point to a valid directory. Make sure to create it first.", argument);
      std::process::exit(1);
    }
  } else if let Ok(path) = std::env::var(env_var) {
    let path = PathBuf::from(path.trim());
    if path.is_dir() {
      return Some(path);
    } else {
      error!("{} env variable was specified, but it doesn't point to a valid directory. Make sure to create it first.", env_var);
      std::process::exit(1);
    }
  } else {
    None
  }
}

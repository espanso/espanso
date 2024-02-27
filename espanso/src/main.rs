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
#![windows_subsystem = "windows"]

use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use cli::{CliAlias, CliModule, CliModuleArgs};
use log::{error, info, warn};
use logging::FileProxy;
use simplelog::{
  CombinedLogger, ConfigBuilder, LevelFilter, SharedLogger, TermLogger, TerminalMode, WriteLogger,
};

use crate::{
  cli::{LogMode, PathsOverrides},
  config::load_config,
  util::log_system_info,
};

mod capabilities;
mod cli;
mod common_flags;
mod config;
mod exit_code;
mod gui;
mod icon;
mod ipc;
mod lock;
#[macro_use]
mod logging;
mod patch;
mod path;
mod preferences;
mod util;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOG_FILE_NAME: &str = "espanso.log";

use lazy_static::lazy_static;

lazy_static! {
  static ref CLI_HANDLERS: Vec<CliModule> = vec![
    cli::path::new(),
    cli::edit::new(),
    cli::launcher::new(),
    cli::log::new(),
    cli::worker::new(),
    cli::daemon::new(),
    cli::modulo::new(),
    cli::migrate::new(),
    cli::env_path::new(),
    cli::service::new(),
    cli::workaround::new(),
    cli::package::new(),
    cli::match_cli::new(),
    cli::cmd::new(),
  ];
  static ref ALIASES: Vec<CliAlias> = vec![
    CliAlias {
      subcommand: "start".to_owned(),
      forward_into: "service".to_owned(),
    },
    CliAlias {
      subcommand: "restart".to_owned(),
      forward_into: "service".to_owned(),
    },
    CliAlias {
      subcommand: "stop".to_owned(),
      forward_into: "service".to_owned(),
    },
    CliAlias {
      subcommand: "status".to_owned(),
      forward_into: "service".to_owned(),
    },
    CliAlias {
      subcommand: "install".to_owned(),
      forward_into: "package".to_owned(),
    },
    CliAlias {
      subcommand: "uninstall".to_owned(),
      forward_into: "package".to_owned(),
    },
  ];
}

fn main() {
  util::attach_console();

  let install_subcommand = SubCommand::with_name("install")
    .about("Install a package")
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
      Arg::with_name("version")
        .long("version")
        .required(false)
        .takes_value(true)
        .help("Force a particular version to be installed instead of the latest available."),
    )
    .arg(
      Arg::with_name("git")
        .long("git")
        .required(false)
        .takes_value(true)
        .help("Git repository from which espanso should install the package."),
    )
    .arg(
      Arg::with_name("git-branch")
        .long("git-branch")
        .required(false)
        .takes_value(true)
        .help("Force espanso to search for the package on a specific git branch"),
    )
    .arg(
      Arg::with_name("force")
        .long("force")
        .required(false)
        .takes_value(false)
        .help("Overwrite the package if already installed"),
    )
    .arg(
      Arg::with_name("refresh-index")
        .long("refresh-index")
        .required(false)
        .takes_value(false)
        .help("Request a fresh copy of the Espanso Hub package index instead of using the cached version.")
    )
    .arg(
      Arg::with_name("use-native-git")
        .long("use-native-git")
        .required(false)
        .takes_value(false)
        .help("If specified, espanso will use the 'git' command instead of trying direct methods."),
    );

  let uninstall_subcommand = SubCommand::with_name("uninstall")
    .about("Remove a package")
    .arg(Arg::with_name("package_name").help("Package name"));

  let start_subcommand = SubCommand::with_name("start")
    .about("Start espanso as a service")
    .arg(
      Arg::with_name("unmanaged")
        .long("unmanaged")
        .required(false)
        .takes_value(false)
        .help("Run espanso as an unmanaged service (avoid system manager)"),
    );
  let restart_subcommand = start_subcommand
    .clone()
    .about("Restart the espanso service")
    .name("restart");
  let stop_subcommand = SubCommand::with_name("stop").about("Stop espanso service");
  let status_subcommand =
    SubCommand::with_name("status").about("Check if the espanso daemon is running or not.");

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
        .hidden(true)
        .help("Specify a custom path from which espanso should read the configuration"),
    )
    .arg(
      Arg::with_name("package_dir")
        .long("package_dir")
        .takes_value(true)
        .hidden(true)
        .help("Specify a custom path for the espanso package directory"),
    )
    .arg(
      Arg::with_name("runtime_dir")
        .long("runtime_dir")
        .takes_value(true)
        .hidden(true)
        .help("Specify a custom path for the espanso runtime directory"),
    )
    .subcommand(
      SubCommand::with_name("env-path")
        .arg(
          Arg::with_name("prompt")
            .long("prompt")
            .required(false)
            .takes_value(false)
            .help("macOS only:Prompt for permissions if the operation requires elevated privileges."),
        )
        .subcommand(SubCommand::with_name("register").about("Add 'espanso' command to PATH"))
        .subcommand(SubCommand::with_name("unregister").about("Remove 'espanso' command from PATH"))
        .about("Add or remove the 'espanso' command from the PATH"),
    )
    .subcommand(SubCommand::with_name("cmd")
        .about("Send a command to the espanso daemon.")
        .subcommand(SubCommand::with_name("enable")
            .about("Enable expansions."))
        .subcommand(SubCommand::with_name("disable")
            .about("Disable expansions."))
        .subcommand(SubCommand::with_name("toggle")
            .about("Enable/Disable expansions."))
        .subcommand(SubCommand::with_name("search")
            .about("Open the Espanso's search bar."))
    )
    .subcommand(SubCommand::with_name("edit")
        .about("Shortcut to open the default text editor to edit config files")
        .arg(Arg::with_name("target_file")
            .help(r#"Defaults to "match/base.yml", it contains the relative path of the file you want to edit, 
such as 'config/default.yml' or 'match/base.yml'. 
For convenience, you can also specify the name directly and Espanso will figure out the path. 
For example, specifying 'email' is equivalent to 'match/email.yml'."#))
        // .arg(Arg::with_name("norestart")
        //     .short("n")
        //     .long("norestart")
        //     .required(false)
        //     .takes_value(false)
        //     .help("Avoid restarting espanso after editing the file"))
    )
    // .subcommand(SubCommand::with_name("detect")
    //     .about("Tool to detect current window properties, to simplify filters creation."))
    .subcommand(
      SubCommand::with_name("daemon")
        .setting(AppSettings::Hidden)
        .about("Start the daemon without spawning a new process."),
    )
    .subcommand(SubCommand::with_name("launcher").setting(AppSettings::Hidden))
    .subcommand(SubCommand::with_name("log").about("Print the daemon logs."))
    .subcommand(
      SubCommand::with_name("modulo")
        .setting(AppSettings::Hidden)
        .subcommand(
          SubCommand::with_name("form")
            .about("Display a customizable form")
            .arg(
              Arg::with_name("input_file")
                .short("i")
                .takes_value(true)
                .help("Input file or - for stdin"),
            )
            .arg(
              Arg::with_name("json")
                .short("j")
                .required(false)
                .takes_value(false)
                .help("Interpret the input data as JSON"),
            ),
        )
        .subcommand(
          SubCommand::with_name("search")
            .about("Display a search box")
            .arg(
              Arg::with_name("input_file")
                .short("i")
                .takes_value(true)
                .help("Input file or - for stdin"),
            )
            .arg(
              Arg::with_name("json")
                .short("j")
                .required(false)
                .takes_value(false)
                .help("Interpret the input data as JSON"),
            ),
        )
        .subcommand(
          SubCommand::with_name("textview")
            .about("Display a Text View")
            .arg(
              Arg::with_name("input_file")
                .short("i")
                .takes_value(true)
                .help("Input file or - for stdin"),
            )
            .arg(
              Arg::with_name("title")
              .long("title")
                .required(true)
                .takes_value(true)
                .help("Window title to display"),
            ),
        )
        .subcommand(SubCommand::with_name("troubleshoot").about("Display the troubleshooting GUI"))
        .subcommand(
          SubCommand::with_name("welcome")
            .about("Display the welcome screen")
            .arg(
              Arg::with_name("already-running")
                .long("already-running")
                .takes_value(false),
            ),
        ),
    )
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
    .subcommand(
      SubCommand::with_name("migrate")
        .about("Automatically migrate legacy config files to the new v2 format.")
        .arg(Arg::with_name("noconfirm").long("noconfirm"))
        .help("Migrate the configuration without asking for confirmation"),
    )
    .subcommand(
      SubCommand::with_name("service")
        .subcommand(SubCommand::with_name("register").about("Register espanso as a system service"))
        .subcommand(
          SubCommand::with_name("unregister").about("Unregister espanso from system services"),
        )
        .subcommand(
          SubCommand::with_name("check")
            .about("Check if espanso is registered as a system service"),
        )
        .subcommand(start_subcommand.clone())
        .subcommand(restart_subcommand.clone())
        .subcommand(stop_subcommand.clone())
        .subcommand(status_subcommand.clone())
        .about("A collection of commands to manage the Espanso service (for example, enabling auto-start on system boot)."),
    )
    .subcommand(start_subcommand)
    .subcommand(restart_subcommand)
    .subcommand(stop_subcommand)
    .subcommand(status_subcommand)
    .subcommand(SubCommand::with_name("match")
        .about("List and execute matches from the CLI")
        .subcommand(SubCommand::with_name("list")
            .about("Print matches to standard output")
            .arg(Arg::with_name("json")
                .short("j")
                .long("json")
                .help("Output matches to the JSON format")
                .required(false)
                .takes_value(false)
            )
            .arg(Arg::with_name("onlytriggers")
                .short("t")
                .long("only-triggers")
                .help("Print only triggers without replacement")
                .required(false)
                .takes_value(false)
            )
            .arg(Arg::with_name("preservenewlines")
                .short("n")
                .long("preserve-newlines")
                .help("Preserve newlines when printing replacements. Does nothing when using JSON format.")
                .required(false)
                .takes_value(false)
            )
            .arg(Arg::with_name("class")
                .long("class")
                .help("Only return matches that would be active with the given class. This is relevant if you want to list matches only active inside an app-specific config.")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("title")
                .long("title")
                .help("Only return matches that would be active with the given title. This is relevant if you want to list matches only active inside an app-specific config.")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("exec")
                .long("exec")
                .help("Only return matches that would be active with the given exec. This is relevant if you want to list matches only active inside an app-specific config.")
                .required(false)
                .takes_value(true)
            )
        )
        .subcommand(SubCommand::with_name("exec")
            .about("Triggers the expansion of a match")
            .arg(Arg::with_name("trigger")
                .short("t")
                .long("trigger")
                .help("The trigger of the match to be expanded")
                .required(false)
                .takes_value(true)
            )
            .arg(Arg::with_name("arg")
                .long("arg")
                .help("Specify also an argument for the expansion, following the --arg 'name=value' format. You can specify multiple ones.")
                .required(false)
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
            )
        )
    )
    .subcommand(
      SubCommand::with_name("package")
        .about("package-management commands")
        .subcommand(install_subcommand.clone())
        .subcommand(uninstall_subcommand.clone())
        .subcommand(SubCommand::with_name("update").about(
          "Update a package. If 'all' is passed as package name, attempts to update all packages.",
        ).arg(Arg::with_name("package_name").help("Package name")))
        .subcommand(
          SubCommand::with_name("list").about("List all installed packages"), // TODO: update <Package> and update all
        ),
    )
    .subcommand(
      SubCommand::with_name("workaround")
        .subcommand(
          SubCommand::with_name("secure-input")
            .about("Attempt to disable secure input by automating the common steps."),
        )
        .about("A collection of workarounds to solve some common problems."),
    )
    .subcommand(
      SubCommand::with_name("worker")
        .setting(AppSettings::Hidden)
        .arg(
          Arg::with_name("start-reason")
            .long("start-reason")
            .required(false)
            .takes_value(true),
        )
        .arg(
          Arg::with_name("monitor-daemon")
            .long("monitor-daemon")
            .required(false)
            .takes_value(false),
        ),
    )
    .subcommand(install_subcommand)
    .subcommand(uninstall_subcommand);

  // TODO: explain that the register and unregister commands are only meaningful
  // when using the system daemon manager on macOS and Linux

  // TODO: set the LSEnvironment variable as described here: https://stackoverflow.com/questions/12203377/combined-gui-and-command-line-os-x-app?rq=1
  // to detect if the executable was launched inside an AppBundle, and if so, launch the "launcher" handler
  // This should only apply when on macOS.

  let matches = clap_instance.clone().get_matches();
  let log_level = match matches.occurrences_of("v") {
    0 | 1 => LevelFilter::Info,

    // Trace mode is only available in debug mode for security reasons
    #[cfg(debug_assertions)]
    3 => LevelFilter::Trace,

    _ => LevelFilter::Debug,
  };

  let alias = ALIASES
    .iter()
    .find(|cli| matches.subcommand_matches(&cli.subcommand).is_some());

  let mut handler = if let Some(alias) = alias {
    CLI_HANDLERS
      .iter()
      .find(|cli| cli.subcommand == alias.forward_into)
  } else {
    CLI_HANDLERS
      .iter()
      .find(|cli| matches.subcommand_matches(&cli.subcommand).is_some())
  };

  if handler.is_none() {
    // When started from the macOS App Bundle, override the default
    // handler with "launcher" if not present, otherwise the GUI could not be started.
    if let Some(context) = std::env::var_os("MAC_LAUNCH_CONTEXT") {
      if context == "bundle" {
        handler = CLI_HANDLERS.iter().find(|cli| cli.subcommand == "launcher");
      }
    }

    // When started from a Linux app image, override the default handler with the launcher
    // to start espanso when launching it directly
    if std::env::var_os("APPIMAGE").is_some() {
      handler = CLI_HANDLERS.iter().find(|cli| cli.subcommand == "launcher");
    }
  }

  if let Some(handler) = handler {
    let log_proxy = FileProxy::new();
    if handler.enable_logs {
      let config = ConfigBuilder::new()
        .set_time_to_local(true)
        .set_time_format(format!(
          "%H:%M:%S [{}({})]",
          handler.subcommand,
          std::process::id()
        ))
        .set_location_level(LevelFilter::Off)
        .add_filter_ignore_str("html5ever")
        .build();

      let mut outputs: Vec<Box<dyn SharedLogger>> = vec![WriteLogger::new(
        LevelFilter::Info,
        config.clone(),
        log_proxy.clone(),
      )];

      if !handler.disable_logs_terminal_output {
        outputs.insert(0, TermLogger::new(log_level, config, TerminalMode::Mixed));
      }

      CombinedLogger::init(outputs).expect("unable to initialize logs");

      // Activate logging for panics
      log_panics::init();
    }

    // If the process doesn't require linux capabilities, disable them
    if !handler.requires_linux_capabilities {
      if let Err(err) = crate::capabilities::clear_capabilities() {
        error!("unable to clear linux capabilities: {}", err);
      }
    }

    // If explicitly requested, we show the Dock icon on macOS
    // We need to enable this selectively, otherwise we would end up with multiple
    // dock icons due to the multi-process nature of espanso.
    #[cfg(target_os = "macos")]
    if handler.show_in_dock {
      espanso_mac_utils::convert_to_foreground_app();
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

      cli_args.paths_overrides = Some(PathsOverrides {
        config: force_config_path,
        packages: force_package_path,
        runtime: force_runtime_path,
      });

      info!("reading configs from: {:?}", paths.config);
      info!("reading packages from: {:?}", paths.packages);
      info!("using runtime dir: {:?}", paths.runtime);
      log_system_info();

      if handler.requires_config {
        let config_result =
          load_config(&paths.config, &paths.packages).expect("unable to load config");

        cli_args.is_legacy_config = config_result.is_legacy_config;
        cli_args.config_store = Some(config_result.config_store);
        cli_args.match_store = Some(config_result.match_store);
        cli_args.non_fatal_errors = config_result.non_fatal_errors;

        if config_result.is_legacy_config {
          warn!("espanso is reading the configuration using compatibility mode, thus some features might not be available");
          warn!("you can migrate to the new configuration format by running 'espanso migrate' in a terminal");
        }
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

    // If the current handler is an alias, rather than sending the sub-arguments
    // we simply forward the current ones
    // For example, the args for "espanso start" are forwarded to "espanso service start"
    if alias.is_some() {
      cli_args.cli_args = Some(matches);
    } else if let Some(args) = matches.subcommand_matches(&handler.subcommand) {
      cli_args.cli_args = Some(args.clone());
    }

    let exit_code = (handler.entry)(cli_args);

    std::process::exit(exit_code);
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
      Some(path)
    } else {
      error_eprintln!("{} argument was specified, but it doesn't point to a valid directory. Make sure to create it first.", argument);
      std::process::exit(1);
    }
  } else if let Ok(path) = std::env::var(env_var) {
    let path = PathBuf::from(path.trim());
    if path.is_dir() {
      Some(path)
    } else {
      error_eprintln!("{} env variable was specified, but it doesn't point to a valid directory. Make sure to create it first.", env_var);
      std::process::exit(1);
    }
  } else {
    None
  }
}

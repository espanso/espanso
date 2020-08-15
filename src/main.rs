/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

 #![cfg_attr(not(test), windows_subsystem = "windows")]

#[macro_use]
extern crate lazy_static;

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::process::exit;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use fs2::FileExt;
use log::{error, info, warn, LevelFilter};
use simplelog::{CombinedLogger, SharedLogger, TermLogger, TerminalMode, WriteLogger};

use crate::config::runtime::RuntimeConfigManager;
use crate::config::{ConfigManager, ConfigSet, Configs};
use crate::engine::Engine;
use crate::event::manager::{DefaultEventManager, EventManager};
use crate::event::*;
use crate::matcher::scrolling::ScrollingMatcher;
use crate::package::default::DefaultPackageManager;
use crate::package::zip::ZipPackageResolver;
use crate::package::{InstallResult, PackageManager, RemoveResult, UpdateResult};
use crate::protocol::*;
use crate::system::SystemManager;
use crate::ui::UIManager;

mod bridge;
mod check;
mod cli;
mod clipboard;
mod config;
mod context;
mod edit;
mod engine;
mod event;
mod extension;
mod keyboard;
mod matcher;
mod package;
mod process;
mod protocol;
mod render;
mod sysdaemon;
mod system;
mod ui;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LOG_FILE: &str = "espanso.log";

fn main() {
    attach_console();

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
        );

    let uninstall_subcommand = SubCommand::with_name("uninstall")
        .about("Remove an installed package. Equivalent to 'espanso package uninstall'")
        .arg(Arg::with_name("package_name").help("Package name"));

    let mut clap_instance = App::new("espanso")
        .version(VERSION)
        .author("Federico Terzi")
        .about("Cross-platform Text Expander written in Rust")
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .subcommand(SubCommand::with_name("cmd")
            .about("Send a command to the espanso daemon.")
            .subcommand(SubCommand::with_name("exit")
                .about("Terminate the daemon."))
            .subcommand(SubCommand::with_name("enable")
                .about("Enable the espanso replacement engine."))
            .subcommand(SubCommand::with_name("disable")
                .about("Disable the espanso replacement engine."))
            .subcommand(SubCommand::with_name("toggle")
                .about("Toggle the status of the espanso replacement engine."))
        )
        .subcommand(SubCommand::with_name("edit")
            .about("Open the default text editor to edit config files and reload them automatically when exiting")
            .arg(Arg::with_name("config")
                .help("Defaults to \"default\". The configuration file name to edit (without the .yml extension)."))
            .arg(Arg::with_name("norestart")
                .short("n")
                .long("norestart")
                .required(false)
                .takes_value(false)
                .help("Avoid restarting espanso after editing the file"))
        )
        .subcommand(SubCommand::with_name("dump")
            .about("Prints all current configuration options."))
        .subcommand(SubCommand::with_name("detect")
            .about("Tool to detect current window properties, to simplify filters creation."))
        .subcommand(SubCommand::with_name("daemon")
            .about("Start the daemon without spawning a new process."))
        .subcommand(SubCommand::with_name("register")
            .about("MacOS and Linux only. Register espanso in the system daemon manager."))
        .subcommand(SubCommand::with_name("unregister")
            .about("MacOS and Linux only. Unregister espanso from the system daemon manager."))
        .subcommand(SubCommand::with_name("log")
            .about("Print the latest daemon logs."))
        .subcommand(SubCommand::with_name("start")
            .about("Start the daemon spawning a new process in the background."))
        .subcommand(SubCommand::with_name("stop")
            .about("Stop the espanso daemon."))
        .subcommand(SubCommand::with_name("restart")
            .about("Restart the espanso daemon."))
        .subcommand(SubCommand::with_name("status")
            .about("Check if the espanso daemon is running or not."))
        .subcommand(SubCommand::with_name("path")
            .about("Prints all the current espanso directory paths, to easily locate configuration and data paths.")
            .subcommand(SubCommand::with_name("config")
                .about("Print the current config folder path."))
            .subcommand(SubCommand::with_name("packages")
                .about("Print the current packages folder path."))
            .subcommand(SubCommand::with_name("data")
                .about("Print the current data folder path."))
            .subcommand(SubCommand::with_name("default")
                .about("Print the default configuration file path."))
        )
        .subcommand(SubCommand::with_name("match")
            .about("List and execute matches from the CLI")
            .subcommand(SubCommand::with_name("list")
                .about("Print all matches to standard output")
                .arg(Arg::with_name("json")
                    .short("j")
                    .long("json")
                    .help("Return the matches as json")
                    .required(false)
                    .takes_value(false)
                )
                .arg(Arg::with_name("onlytriggers")
                    .short("t")
                    .long("onlytriggers")
                    .help("Print only triggers without replacement")
                    .required(false)
                    .takes_value(false)
                )
                .arg(Arg::with_name("preservenewlines")
                    .short("n")
                    .long("preservenewlines")
                    .help("Preserve newlines when printing replacements")
                    .required(false)
                    .takes_value(false)
                )
            )
            .subcommand(SubCommand::with_name("exec")
                .about("Triggers the expansion of the given match")
                .arg(Arg::with_name("trigger")
                    .help("The trigger of the match to be expanded")
                )
            )
        )
        // Package manager
        .subcommand(SubCommand::with_name("package")
            .about("Espanso package manager commands")
            .subcommand(install_subcommand.clone())
            .subcommand(uninstall_subcommand.clone())
            .subcommand(SubCommand::with_name("list")
                .about("List all installed packages")
                .arg(Arg::with_name("full")
                    .help("Print all package info")
                    .long("full")))

            .subcommand(SubCommand::with_name("refresh")
                .about("Update espanso package index"))
        )
        .subcommand(SubCommand::with_name("worker")
            .setting(AppSettings::Hidden)
            .arg(Arg::with_name("reload")
                .short("r")
                .long("reload")
                .required(false)
                .takes_value(false))
        )
        .subcommand(install_subcommand)
        .subcommand(uninstall_subcommand);

    let matches = clap_instance.clone().get_matches();

    // The edit subcommand must be run before the configuration parsing. Otherwise, if the
    // configuration is corrupted, the edit command won't work, which makes it pretty useless.
    if let Some(matches) = matches.subcommand_matches("edit") {
        edit_main(matches);
        return;
    }

    let log_level = matches.occurrences_of("v") as i32;

    // Load the configuration
    let mut config_set = ConfigSet::load_default().unwrap_or_else(|e| {
        println!("{}", e);
        exit(1);
    });

    config_set.default.log_level = log_level;

    // Commands that require the configuration

    if let Some(matches) = matches.subcommand_matches("cmd") {
        cmd_main(config_set, matches);
        return;
    }

    if matches.subcommand_matches("dump").is_some() {
        println!("{:#?}", config_set);
        return;
    }

    if matches.subcommand_matches("detect").is_some() {
        detect_main();
        return;
    }

    if matches.subcommand_matches("daemon").is_some() {
        daemon_main(config_set);
        return;
    }

    if matches.subcommand_matches("register").is_some() {
        register_main(config_set);
        return;
    }

    if matches.subcommand_matches("unregister").is_some() {
        unregister_main(config_set);
        return;
    }

    if matches.subcommand_matches("log").is_some() {
        log_main();
        return;
    }

    if matches.subcommand_matches("start").is_some() {
        start_main(config_set);
        return;
    }

    if matches.subcommand_matches("status").is_some() {
        status_main();
        return;
    }

    if matches.subcommand_matches("stop").is_some() {
        stop_main(config_set);
        return;
    }

    if matches.subcommand_matches("restart").is_some() {
        restart_main(config_set);
        return;
    }

    if let Some(matches) = matches.subcommand_matches("install") {
        install_main(config_set, matches);
        return;
    }

    if let Some(matches) = matches.subcommand_matches("uninstall") {
        remove_package_main(config_set, matches);
        return;
    }

    if let Some(matches) = matches.subcommand_matches("path") {
        path_main(config_set, matches);
        return;
    }

    if let Some(matches) = matches.subcommand_matches("match") {
        match_main(config_set, matches);
        return;
    }

    if let Some(matches) = matches.subcommand_matches("package") {
        if let Some(matches) = matches.subcommand_matches("install") {
            install_main(config_set, matches);
            return;
        }
        if let Some(matches) = matches.subcommand_matches("uninstall") {
            remove_package_main(config_set, matches);
            return;
        }
        if let Some(matches) = matches.subcommand_matches("list") {
            list_package_main(config_set, matches);
            return;
        }
        if matches.subcommand_matches("refresh").is_some() {
            update_index_main(config_set);
            return;
        }
    }

    if let Some(matches) = matches.subcommand_matches("worker") {
        worker_main(config_set, matches);
        return;
    }

    // Defaults help print
    clap_instance
        .print_long_help()
        .expect("Unable to print help");
    println!();
}

#[cfg(target_os = "windows")]
fn attach_console() {
    // When using the windows subsystem we loose the terminal output.
    // Therefore we try to attach to the current console if available.
    unsafe {winapi::um::wincon::AttachConsole(0xFFFFFFFF)};
}

#[cfg(not(target_os = "windows"))]
fn attach_console() {
    // Not necessary on Linux and macOS
}

fn init_logger(config_set: &ConfigSet, reset: bool) {
    // Initialize log
    let log_level = match config_set.default.log_level {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 | _ => LevelFilter::Debug,
    };

    let mut log_outputs: Vec<Box<dyn SharedLogger>> = Vec::new();

    // Initialize terminal output
    let terminal_out =
        TermLogger::new(log_level, simplelog::Config::default(), TerminalMode::Mixed);
    if let Some(terminal_out) = terminal_out {
        log_outputs.push(terminal_out);
    }

    // Initialize log file output
    let espanso_dir = context::get_data_dir();
    let log_file_path = espanso_dir.join(LOG_FILE);

    if reset && log_file_path.exists() {
        std::fs::remove_file(&log_file_path).expect("unable to remove log file");
    }

    let log_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(log_file_path)
        .expect("Cannot create log file.");
    let file_out = WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), log_file);
    log_outputs.push(file_out);

    CombinedLogger::init(log_outputs).expect("Error opening log destination");

    // Activate logging for panics
    log_panics::init();
}

/// Daemon subcommand, start the event loop and spawn a background thread worker
fn daemon_main(config_set: ConfigSet) {
    // Try to acquire lock file
    let lock_file = acquire_lock();
    if lock_file.is_none() {
        println!("espanso is already running.");
        exit(3);
    }

    precheck_guard();

    init_logger(&config_set, true);

    info!("espanso version {}", VERSION);
    info!(
        "using config path: {}",
        context::get_config_dir().to_string_lossy()
    );
    info!(
        "using package path: {}",
        context::get_package_dir().to_string_lossy()
    );

    let (send_channel, receive_channel) = mpsc::channel();

    let ipc_server = protocol::get_ipc_server(
        Service::Daemon,
        config_set.default.clone(),
        send_channel.clone(),
    );
    ipc_server.start();

    info!("spawning worker process...");

    let espanso_path = std::env::current_exe().expect("unable to obtain espanso path location");
    let mut child = crate::process::spawn_process(
        &espanso_path.to_string_lossy().to_string(),
        &vec!["worker".to_owned()],
    )
    .expect("unable to create worker process");

    // Create a monitor thread that will exit with the same non-zero code if
    // the worker thread exits
    thread::Builder::new()
        .name("worker monitor".to_string())
        .spawn(move || {
            let result = child.wait();
            if let Ok(status) = result {
                if let Some(code) = status.code() {
                    if code != 0 {
                        error!(
                            "worker process exited with non-zero code: {}, exiting",
                            code
                        );
                        std::process::exit(code);
                    }
                }
            }
        })
        .expect("Unable to spawn worker monitor thread");

    register_signals(config_set.default.clone());

    std::thread::sleep(Duration::from_millis(200));

    if config_set.default.auto_restart {
        let send_channel_clone = send_channel.clone();
        thread::Builder::new()
            .name("watcher_background".to_string())
            .spawn(move || {
                watcher_background(send_channel_clone);
            })
            .expect("Unable to spawn watcher background thread");
    }

    loop {
        match receive_channel.recv() {
            Ok(event) => {
                match event {
                    Event::Action(ActionType::RestartWorker) => {
                        // Terminate the worker process
                        send_command_or_warn(
                            Service::Worker,
                            config_set.default.clone(),
                            IPCCommand::exit_worker(),
                        );

                        std::thread::sleep(Duration::from_millis(500));

                        // Restart the worker process
                        crate::process::spawn_process(
                            &espanso_path.to_string_lossy().to_string(),
                            &vec!["worker".to_owned(), "--reload".to_owned()],
                        );
                    }
                    Event::Action(ActionType::Exit) => {
                        send_command_or_warn(
                            Service::Worker,
                            config_set.default.clone(),
                            IPCCommand::exit_worker(),
                        );

                        std::thread::sleep(Duration::from_millis(200));

                        info!("terminating espanso.");
                        std::process::exit(0);
                    }
                    _ => {
                        // Forward the command to the worker
                        let command = IPCCommand::from(event);
                        if let Some(command) = command {
                            send_command_or_warn(
                                Service::Worker,
                                config_set.default.clone(),
                                command,
                            );
                        }
                    }
                }
            }
            Err(e) => {
                warn!("error while reading event in daemon process: {}", e);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn register_signals(_: Configs) {}

#[cfg(not(target_os = "windows"))]
fn register_signals(config: Configs) {
    // On Unix, also listen for signals so that we can terminate the
    // worker if the daemon receives a signal
    use signal_hook::{iterator::Signals, SIGINT, SIGTERM};
    let signals = Signals::new(&[SIGTERM, SIGINT]).expect("unable to register for signals");
    thread::Builder::new()
        .name("signal monitor".to_string())
        .spawn(move || {
            for signal in signals.forever() {
                info!("Received signal: {:?}, terminating worker", signal);
                send_command_or_warn(Service::Worker, config, IPCCommand::exit_worker());

                std::thread::sleep(Duration::from_millis(200));

                info!("terminating espanso.");
                std::process::exit(0);
            }
        })
        .expect("Unable to spawn signal monitor thread");
}

fn watcher_background(sender: Sender<Event>) {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher =
        Watcher::new(tx, Duration::from_secs(1)).expect("unable to create file watcher");

    let config_path = crate::context::get_config_dir();
    watcher
        .watch(&config_path, RecursiveMode::Recursive)
        .expect("unable to start watcher");

    info!(
        "watching for changes in path: {}",
        config_path.to_string_lossy()
    );

    loop {
        let should_reload = match rx.recv() {
            Ok(event) => {
                let path = match event {
                    DebouncedEvent::Create(path) => Some(path),
                    DebouncedEvent::Write(path) => Some(path),
                    DebouncedEvent::Remove(path) => Some(path),
                    DebouncedEvent::Rename(_, path) => Some(path),
                    _ => None,
                };

                if let Some(path) = path {
                    if path.extension().unwrap_or_default() == "yml" && 
                      !path.file_name().unwrap_or_default().to_string_lossy().starts_with("."){
                        // Only load non-hidden yml files
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Err(e) => {
                warn!("error while watching files: {:?}", e);
                false
            }
        };

        if should_reload {
            info!("change detected, restarting worker process...");

            let mut config_set = ConfigSet::load_default();

            match config_set {
                Ok(config_set) => {
                    let event = Event::Action(ActionType::RestartWorker);
                    sender.send(event).unwrap_or_else(|e| {
                        warn!("unable to communicate with daemon thread: {}", e);
                    })
                }
                Err(error) => {
                    error!("Unable to reload configuration due to an error: {}", error);
                    let event = Event::System(SystemEvent::NotifyRequest(
                        "Unable to reload config due to an error, see the logs for more details."
                            .to_owned(),
                    ));
                    sender.send(event).unwrap_or_else(|e| {
                        warn!("unable to communicate with daemon thread: {}", e);
                    })
                }
            }
        }
    }
}

/// Worker process main which does the actual work
fn worker_main(config_set: ConfigSet, matches: &ArgMatches) {
    init_logger(&config_set, false);

    info!("initializing worker process...");

    let is_reloading: bool = if matches.is_present("reload") {
        true
    } else {
        false
    };

    let (send_channel, receive_channel) = mpsc::channel();

    // This atomic bool is used to "disable" espanso when espanding its own matches, otherwise
    // we could reinterpret the characters we are injecting
    let is_injecting = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let context = context::new(
        config_set.default.clone(),
        send_channel.clone(),
        is_injecting.clone(),
    );

    let config_set_copy = config_set.clone();
    thread::Builder::new()
        .name("daemon_background".to_string())
        .spawn(move || {
            worker_background(receive_channel, config_set_copy, is_injecting, is_reloading);
        })
        .expect("Unable to spawn daemon background thread");

    let ipc_server =
        protocol::get_ipc_server(Service::Worker, config_set.default, send_channel.clone());
    ipc_server.start();

    context.eventloop();
}

/// Background thread worker for the daemon
fn worker_background(
    receive_channel: Receiver<Event>,
    config_set: ConfigSet,
    is_injecting: Arc<AtomicBool>,
    is_reloading: bool,
) {
    let system_manager = system::get_manager();
    let config_manager = RuntimeConfigManager::new(config_set, system_manager);

    let ui_manager = ui::get_uimanager();
    if config_manager.default_config().show_notifications {
        if !is_reloading {
            ui_manager.notify("espanso is running!");
        } else {
            ui_manager.notify("Reloaded config!");
        }
    }

    let clipboard_manager = clipboard::get_manager();

    let keyboard_manager = keyboard::get_manager();

    let extensions = extension::get_extensions(config_manager.default_config(), Box::new(clipboard::get_manager()));

    let renderer =
        render::default::DefaultRenderer::new(extensions, config_manager.default_config().clone());

    let engine = Engine::new(
        &keyboard_manager,
        &clipboard_manager,
        &config_manager,
        &ui_manager,
        &renderer,
        is_injecting,
    );

    let matcher = ScrollingMatcher::new(&config_manager, &engine);

    let event_manager = DefaultEventManager::new(
        receive_channel,
        vec![&matcher],
        vec![&engine, &matcher],
        vec![&engine],
    );

    info!("worker is running!");

    event_manager.eventloop();
}

/// start subcommand, spawn a background espanso process.
fn start_main(config_set: ConfigSet) {
    // Try to acquire lock file
    let lock_file = acquire_lock();
    if lock_file.is_none() {
        println!("espanso is already running.");
        exit(3);
    }
    release_lock(lock_file.unwrap());

    precheck_guard();

    start_daemon(config_set);
}

#[cfg(target_os = "windows")]
fn start_daemon(_: ConfigSet) {
    unsafe {
        let res = bridge::windows::start_daemon_process();
        if res < 0 {
            println!("Error starting daemon process");
        }
    }
}

#[cfg(target_os = "macos")]
fn start_daemon(config_set: ConfigSet) {
    if config_set.default.use_system_agent {
        use std::process::Command;

        let res = Command::new("launchctl")
            .args(&["start", "com.federicoterzi.espanso"])
            .status();

        if let Ok(status) = res {
            if status.success() {
                println!("Daemon started correctly!")
            } else {
                eprintln!("Error starting launchd daemon with status: {}", status);
            }
        } else {
            eprintln!("Error starting launchd daemon: {}", res.unwrap_err());
        }
    } else {
        fork_daemon(config_set);
    }
}

#[cfg(target_os = "linux")]
fn start_daemon(config_set: ConfigSet) {
    use crate::sysdaemon::{verify, VerifyResult};
    use std::process::{Command, Stdio};

    // Check if Systemd is available in the system
    let status = Command::new("systemctl")
        .args(&["--version"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .status();

    // If Systemd is not available in the system, espanso should default to unmanaged mode
    // See issue https://github.com/federico-terzi/espanso/issues/139
    let force_unmanaged = if let Err(_) = status { true } else { false };

    if config_set.default.use_system_agent && !force_unmanaged {
        // Make sure espanso is currently registered in systemd
        let res = verify();
        match res {
            VerifyResult::EnabledAndValid => {
                // Do nothing, everything is ok!
            }
            VerifyResult::EnabledButInvalidPath => {
                eprintln!("Updating espanso service file with new path...");
                unregister_main(config_set.clone());
                register_main(config_set);
            }
            VerifyResult::NotEnabled => {
                use dialoguer::Confirmation;
                if Confirmation::new()
                    .with_text("espanso must be registered to systemd (user level) first. Do you want to proceed?")
                    .default(true)
                    .show_default(true)
                    .interact().expect("Unable to read user answer") {

                    register_main(config_set);
                }else{
                    eprintln!("Please register espanso to systemd with this command:");
                    eprintln!("   espanso register");
                    // TODO: enable flag to use non-managed daemon mode

                    std::process::exit(4);
                }
            }
        }

        // Start the espanso service
        let res = Command::new("systemctl")
            .args(&["--user", "start", "espanso.service"])
            .status();

        if let Ok(status) = res {
            if status.success() {
                println!("Daemon started correctly!")
            } else {
                eprintln!("Error starting systemd daemon with status: {}", status);
            }
        } else {
            eprintln!("Error starting systemd daemon: {}", res.unwrap_err());
        }
    } else {
        if force_unmanaged {
            eprintln!("Systemd is not available in this system, switching to unmanaged mode.");
        }

        fork_daemon(config_set);
    }
}

#[cfg(not(target_os = "windows"))]
fn fork_daemon(config_set: ConfigSet) {
    unsafe {
        let pid = libc::fork();
        if pid < 0 {
            println!("Unable to fork.");
            exit(4);
        }
        if pid > 0 {
            // Parent process exit
            println!("daemon started!");
            exit(0);
        }

        // Spawned process

        // Create a new SID for the child process
        let sid = libc::setsid();
        if sid < 0 {
            exit(5);
        }

        // Detach stdout and stderr
        let null_path = std::ffi::CString::new("/dev/null").expect("CString unwrap failed");
        let fd = libc::open(null_path.as_ptr(), libc::O_RDWR, 0);
        if fd != -1 {
            libc::dup2(fd, libc::STDIN_FILENO);
            libc::dup2(fd, libc::STDOUT_FILENO);
            libc::dup2(fd, libc::STDERR_FILENO);
        }
    }

    daemon_main(config_set);
}

/// status subcommand, print the current espanso status
fn status_main() {
    let lock_file = acquire_lock();
    if let Some(lock_file) = lock_file {
        println!("espanso is not running");

        release_lock(lock_file);
    } else {
        println!("espanso is running");
    }
}

/// Stop subcommand, used to stop the daemon.
fn stop_main(config_set: ConfigSet) {
    // Try to acquire lock file
    let lock_file = acquire_lock();
    if lock_file.is_some() {
        println!("espanso daemon is not running.");
        release_lock(lock_file.unwrap());
        exit(3);
    }

    send_command_or_warn(Service::Daemon, config_set.default, IPCCommand::exit());
}

/// Kill the daemon if running and start it again
fn restart_main(config_set: ConfigSet) {
    // Kill the daemon if running
    let lock_file = acquire_lock();
    if lock_file.is_none() {
        // Terminate the current espanso daemon
        send_command_or_warn(
            Service::Daemon,
            config_set.default.clone(),
            IPCCommand::exit(),
        );
    } else {
        release_lock(lock_file.unwrap());
    }

    std::thread::sleep(Duration::from_millis(500));

    // Restart the daemon
    start_main(config_set);
}

/// Cli tool used to analyze active windows to extract useful information
/// to create configuration filters.
#[cfg(not(target_os = "macos"))]
fn detect_main() {
    let system_manager = system::get_manager();

    println!("Listening for changes, now focus the window you want to analyze.");
    println!("You can terminate with CTRL+C\n");

    let mut last_title: String = "".to_owned();
    let mut last_class: String = "".to_owned();
    let mut last_exec: String = "".to_owned();

    loop {
        let curr_title = system_manager
            .get_current_window_title()
            .unwrap_or_default();
        let curr_class = system_manager
            .get_current_window_class()
            .unwrap_or_default();
        let curr_exec = system_manager
            .get_current_window_executable()
            .unwrap_or_default();

        // Check if a change occurred
        if curr_title != last_title || curr_class != last_class || curr_exec != last_exec {
            println!("Detected change, current window has properties:");
            println!("==> Title: '{}'", curr_title);
            println!("==> Class: '{}'", curr_class);
            println!("==> Executable: '{}'", curr_exec);
            println!();
        }

        last_title = curr_title;
        last_class = curr_class;
        last_exec = curr_exec;

        thread::sleep(Duration::from_millis(500));
    }
}

/// Cli tool used to analyze active windows to extract useful information
/// to create configuration filters.
/// On macOS version we need to start an event loop for the app to register changes.
#[cfg(target_os = "macos")]
fn detect_main() {
    thread::spawn(|| {
        use std::io::stdout;
        use std::io::Write;

        let system_manager = system::get_manager();

        println!("Listening for changes, now focus the window you want to analyze.");
        println!(
            "Warning: stay on the window for a few seconds, as it may take a while to register."
        );
        println!("You can terminate with CTRL+C\n");

        let mut last_title: String = "".to_owned();
        let mut last_class: String = "".to_owned();
        let mut last_exec: String = "".to_owned();

        loop {
            let curr_title = system_manager
                .get_current_window_title()
                .unwrap_or_default();
            let curr_class = system_manager
                .get_current_window_class()
                .unwrap_or_default();
            let curr_exec = system_manager
                .get_current_window_executable()
                .unwrap_or_default();

            // Check if a change occurred
            if curr_title != last_title || curr_class != last_class || curr_exec != last_exec {
                println!("Detected change, current window has properties:");
                println!("==> Title: '{}'", curr_title);
                println!("==> Class: '{}'", curr_class);
                println!("==> Executable: '{}'", curr_exec);
                println!();
            }

            last_title = curr_title;
            last_class = curr_class;
            last_exec = curr_exec;

            thread::sleep(Duration::from_millis(500));
        }
    });

    unsafe {
        crate::bridge::macos::headless_eventloop();
    }
}

/// Send the given command to the espanso daemon
fn cmd_main(config_set: ConfigSet, matches: &ArgMatches) {
    let command = if matches.subcommand_matches("exit").is_some() {
        Some(IPCCommand::exit())
    } else if matches.subcommand_matches("toggle").is_some() {
        Some(IPCCommand {
            id: String::from("toggle"),
            payload: String::from(""),
        })
    } else if matches.subcommand_matches("enable").is_some() {
        Some(IPCCommand {
            id: String::from("enable"),
            payload: String::from(""),
        })
    } else if matches.subcommand_matches("disable").is_some() {
        Some(IPCCommand {
            id: String::from("disable"),
            payload: String::from(""),
        })
    } else {
        None
    };

    if let Some(command) = command {
        send_command_or_warn(Service::Daemon, config_set.default, command);
    }

    exit(1);
}

fn log_main() {
    let espanso_dir = context::get_data_dir();
    let log_file_path = espanso_dir.join(LOG_FILE);

    if !log_file_path.exists() {
        println!("No log file found.");
        exit(2);
    }

    let log_file = File::open(log_file_path);
    if let Ok(log_file) = log_file {
        let reader = BufReader::new(log_file);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
            }
        }

        exit(0);
    } else {
        println!("Error reading log file");
        exit(1);
    }
}

fn register_main(config_set: ConfigSet) {
    sysdaemon::register(config_set);
}

fn unregister_main(config_set: ConfigSet) {
    sysdaemon::unregister(config_set);
}

fn install_main(_config_set: ConfigSet, matches: &ArgMatches) {
    let package_name = matches.value_of("package_name").unwrap_or_else(|| {
        eprintln!("Missing package name!");
        exit(1);
    });

    let mut repository = matches.value_of("repository_url").unwrap_or("hub");

    // Remove trailing .git string if present
    // See: https://github.com/federico-terzi/espanso/issues/326
    if repository.ends_with(".git") {
        repository = repository.trim_end_matches(".git")
    }

    let package_resolver = Box::new(ZipPackageResolver::new());

    let allow_external: bool = if matches.is_present("external") {
        println!("Allowing external repositories");
        true
    } else {
        false
    };

    let mut package_manager = DefaultPackageManager::new_default(Some(package_resolver));

    let res = if repository == "hub" {
        // Installation from the Hub
        if package_manager.is_index_outdated() {
            println!("Updating package index...");
            let res = package_manager.update_index(false);

            match res {
                Ok(update_result) => match update_result {
                    UpdateResult::NotOutdated => {
                        eprintln!("Index was already up to date");
                    }
                    UpdateResult::Updated => {
                        println!("Index updated!");
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                    exit(2);
                }
            }
        } else {
            println!("Using cached package index, run 'espanso package refresh' to update it.")
        }

        package_manager.install_package(package_name, allow_external)
    } else {
        // Make sure the repo is a valid github url
        lazy_static! {
            static ref GITHUB_REGEX: Regex = Regex::new(r#"https://github\.com/\S*/\S*"#).unwrap();
        };

        if !GITHUB_REGEX.is_match(repository) {
            eprintln!("repository url is not valid, it should be an HTTPS GitHub url in the following format:");
            eprintln!("https://github.com/user/repo");
            exit(3);
        }

        if !allow_external {
            Ok(InstallResult::BlockedExternalPackage(repository.to_owned()))
        } else {
            package_manager.install_package_from_repo(package_name, repository)
        }
    };

    match res {
        Ok(install_result) => match install_result {
            InstallResult::NotFoundInIndex => {
                eprintln!("Package not found");
            }
            InstallResult::NotFoundInRepo => {
                eprintln!(
                    "Package not found in repository, are you sure the folder exist in the repo?"
                );
            }
            InstallResult::UnableToParsePackageInfo => {
                eprintln!("Unable to parse Package info from README.md");
            }
            InstallResult::MissingPackageVersion => {
                eprintln!("Missing package version");
            }
            InstallResult::AlreadyInstalled => {
                eprintln!("{} already installed!", package_name);
            }
            InstallResult::BlockedExternalPackage(repo_url) => {
                eprintln!("Warning: the requested package is hosted on an external repository:");
                eprintln!();
                eprintln!("{}", repo_url);
                eprintln!();
                eprintln!("and its contents may not have been verified by espanso.");
                eprintln!();
                eprintln!("For your security, espanso blocks packages that are not verified.");
                eprintln!("If you want to install the package anyway, you can force espanso");
                eprintln!("to install it with the following command, but please do it only");
                eprintln!("if you trust the source or you verified the contents of the package");
                eprintln!("by checking out the repository listed above.");
                eprintln!();

                if repository == "hub" {
                    eprintln!("espanso install {} --external", package_name);
                } else {
                    eprintln!("espanso install {} {} --external", package_name, repository);
                }
                eprintln!();
            }
            InstallResult::Installed => {
                println!("{} successfully installed!", package_name);
                println!();
                println!("You need to restart espanso for changes to take effect, using:");
                println!("  espanso restart");
            }
        },
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

fn remove_package_main(_config_set: ConfigSet, matches: &ArgMatches) {
    let package_name = matches.value_of("package_name").unwrap_or_else(|| {
        eprintln!("Missing package name!");
        exit(1);
    });

    let package_manager = DefaultPackageManager::new_default(None);

    let res = package_manager.remove_package(package_name);

    match res {
        Ok(remove_result) => match remove_result {
            RemoveResult::NotFound => {
                eprintln!("{} package was not installed.", package_name);
            }
            RemoveResult::Removed => {
                println!("{} successfully removed!", package_name);
                println!();
                println!("You need to restart espanso for changes to take effect, using:");
                println!("  espanso restart");
            }
        },
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

fn update_index_main(_config_set: ConfigSet) {
    let mut package_manager = DefaultPackageManager::new_default(None);

    let res = package_manager.update_index(true);

    match res {
        Ok(update_result) => match update_result {
            UpdateResult::NotOutdated => {
                eprintln!("Index was already up to date");
            }
            UpdateResult::Updated => {
                println!("Index updated!");
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            exit(2);
        }
    }
}

fn list_package_main(_config_set: ConfigSet, matches: &ArgMatches) {
    let package_manager = DefaultPackageManager::new_default(None);

    let list = package_manager.list_local_packages();

    if matches.is_present("full") {
        for package in list.iter() {
            println!("{:?}", package);
        }
    } else {
        for package in list.iter() {
            println!("{} - {}", package.name, package.version);
        }
    }
}

fn path_main(_config_set: ConfigSet, matches: &ArgMatches) {
    let config = crate::context::get_config_dir();
    let packages = crate::context::get_package_dir();
    let data = crate::context::get_data_dir();

    if matches.subcommand_matches("config").is_some() {
        println!("{}", config.to_string_lossy());
    } else if matches.subcommand_matches("packages").is_some() {
        println!("{}", packages.to_string_lossy());
    } else if matches.subcommand_matches("data").is_some() {
        println!("{}", data.to_string_lossy());
    } else if matches.subcommand_matches("default").is_some() {
        let default_file = config.join(crate::config::DEFAULT_CONFIG_FILE_NAME);
        println!("{}", default_file.to_string_lossy());
    } else {
        println!("Config: {}", config.to_string_lossy());
        println!("Packages: {}", packages.to_string_lossy());
        println!("Data: {}", data.to_string_lossy());
    }
}

fn match_main(config_set: ConfigSet, matches: &ArgMatches) {
    if let Some(matches) = matches.subcommand_matches("list") {
        let json = matches.is_present("json");
        let onlytriggers = matches.is_present("onlytriggers");
        let preserve_newlines = matches.is_present("preservenewlines");

        if !json {
            crate::cli::list_matches(config_set, onlytriggers, preserve_newlines);
        } else {
            crate::cli::list_matches_as_json(config_set);
        }
    } else if let Some(matches) = matches.subcommand_matches("exec") {
        let trigger = matches.value_of("trigger").unwrap_or_else(|| {
            eprintln!("missing trigger");
            exit(1);
        });

        send_command_or_warn(
            Service::Worker,
            config_set.default.clone(),
            IPCCommand::trigger(trigger),
        );
    }
}

fn edit_main(matches: &ArgMatches) {
    // Determine which is the file to edit
    let config = matches.value_of("config").unwrap_or("default");

    let config_dir = crate::context::get_config_dir();

    let config_path = match config {
        "default" => config_dir.join(crate::config::DEFAULT_CONFIG_FILE_NAME),
        name => {
            // Otherwise, search in the user/ config folder
            config_dir
                .join(crate::config::USER_CONFIGS_FOLDER_NAME)
                .join(name.to_owned() + ".yml")
        }
    };

    println!("Editing file: {:?}", &config_path);

    // Based on the fact that the file already exists or not, we should detect in different
    // ways if a reload is needed
    let should_reload = if config_path.exists() {
        // Get the last modified date, so that we can detect if the user actually edits the file
        // before reloading
        let metadata = std::fs::metadata(&config_path).expect("cannot gather file metadata");
        let last_modified = metadata
            .modified()
            .expect("cannot read file last modified date");

        let result = crate::edit::open_editor(&config_path);
        if result {
            let new_metadata =
                std::fs::metadata(&config_path).expect("cannot gather file metadata");
            let new_last_modified = new_metadata
                .modified()
                .expect("cannot read file last modified date");

            if last_modified != new_last_modified {
                println!("File has been modified, reloading configuration");
                true
            } else {
                println!("File has not been modified, avoiding reload");
                false
            }
        } else {
            false
        }
    } else {
        let result = crate::edit::open_editor(&config_path);
        if result {
            // If the file has been created, we should reload the espanso config
            if config_path.exists() {
                println!("A new file has been created, reloading configuration");
                true
            } else {
                println!("No file has been created, avoiding reload");
                false
            }
        } else {
            false
        }
    };

    let no_restart: bool = if matches.is_present("norestart") {
        println!("Avoiding automatic restart");
        true
    } else {
        false
    };

    if should_reload && !no_restart {
        // Load the configuration
        let config_set = ConfigSet::load_default().unwrap_or_else(|e| {
            eprintln!("{}", e);
            eprintln!("Unable to reload espanso due to previous configuration error.");
            exit(1);
        });

        restart_main(config_set)
    }
}

fn acquire_lock() -> Option<File> {
    acquire_custom_lock("espanso.lock")
}

fn acquire_custom_lock(name: &str) -> Option<File> {
    let espanso_dir = context::get_data_dir();
    let lock_file_path = espanso_dir.join(name);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(lock_file_path)
        .expect("Cannot create reference to lock file.");

    let res = file.try_lock_exclusive();

    if res.is_ok() {
        return Some(file);
    }

    None
}

fn release_lock(lock_file: File) {
    lock_file.unlock().unwrap()
}

/// Used to make sure all the required dependencies and conditions are satisfied before starting espanso.
fn precheck_guard() {
    let satisfied = check::check_preconditions();
    if !satisfied {
        println!();
        println!("Pre-check was not successful, espanso could not be started.");
        exit(5);
    }
}

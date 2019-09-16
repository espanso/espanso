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

#[macro_use]
extern crate lazy_static;

use std::thread;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process::exit;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use clap::{App, Arg, SubCommand, ArgMatches};
use fs2::FileExt;
use log::{info, warn, LevelFilter};
use simplelog::{CombinedLogger, SharedLogger, TerminalMode, TermLogger, WriteLogger};

use crate::config::ConfigSet;
use crate::config::runtime::RuntimeConfigManager;
use crate::engine::Engine;
use crate::event::*;
use crate::event::manager::{DefaultEventManager, EventManager};
use crate::matcher::scrolling::ScrollingMatcher;
use crate::system::SystemManager;
use crate::ui::UIManager;
use crate::protocol::*;
use std::io::{BufReader, BufRead};

mod ui;
mod event;
mod check;
mod bridge;
mod engine;
mod config;
mod system;
mod install;
mod context;
mod matcher;
mod keyboard;
mod protocol;
mod clipboard;
mod extension;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const LOG_FILE: &str = "espanso.log";

fn main() {
    let matches = App::new("espanso")
        .version(VERSION)
        .author("Federico Terzi")
        .about("Cross-platform Text Expander written in Rust")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config directory. If not specified, reads the default $HOME/.espanso/default.yaml file, creating it if not present.")
            .takes_value(true))
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
        .subcommand(SubCommand::with_name("dump")
            .about("Prints all current configuration options."))
        .subcommand(SubCommand::with_name("detect")
            .about("Tool to detect current window properties, to simplify filters creation."))
        .subcommand(SubCommand::with_name("daemon")
            .about("Start the daemon without spawning a new process."))
        .subcommand(SubCommand::with_name("install")
            .about("MacOS and Linux only. Register espanso in the system daemon manager."))
        .subcommand(SubCommand::with_name("uninstall")
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
        .get_matches();

    let log_level = matches.occurrences_of("v") as i32;

    // Load the configuration
    let mut config_set = match matches.value_of("config") {
        None => {
            if log_level > 1 {
                println!("loading configuration from default location...");
            }
            ConfigSet::load_default()
        },
        Some(path) => {
            if log_level > 1 {
                println!("loading configuration from custom location: {}", path);
            }
            ConfigSet::load(Path::new(path))
        },
    }.unwrap_or_else(|e| {
        println!("{}", e);
        exit(1);
    });

    config_set.default.log_level = log_level;

    // Match the correct subcommand

    if let Some(matches) = matches.subcommand_matches("cmd") {
        cmd_main(config_set, matches);
        return;
    }

    if let Some(_) = matches.subcommand_matches("dump") {
        println!("{:#?}", config_set);
        return;
    }

    if let Some(_) = matches.subcommand_matches("detect") {
        detect_main();
        return;
    }

    if let Some(_) = matches.subcommand_matches("daemon") {
        daemon_main(config_set);
        return;
    }

    if let Some(_) = matches.subcommand_matches("install") {
        install_main(config_set);
        return;
    }

    if let Some(_) = matches.subcommand_matches("uninstall") {
        uninstall_main(config_set);
        return;
    }

    if let Some(_) = matches.subcommand_matches("log") {
        log_main();
        return;
    }

    if let Some(_) = matches.subcommand_matches("start") {
        start_main(config_set);
        return;
    }

    if let Some(_) = matches.subcommand_matches("status") {
        status_main();
        return;
    }

    if let Some(_) = matches.subcommand_matches("stop") {
        stop_main(config_set);
        return;
    }

    if let Some(_) = matches.subcommand_matches("restart") {
        restart_main(config_set);
        return;
    }

    // Defaults to start subcommand
    start_main(config_set);
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

    // Initialize log
    let log_level = match config_set.default.log_level {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 | _ => LevelFilter::Debug,
    };

    let mut log_outputs: Vec<Box<dyn SharedLogger>> = Vec::new();

    // Initialize terminal output
    let terminal_out = TermLogger::new(log_level,
                                       simplelog::Config::default(), TerminalMode::Mixed);
    if let Some(terminal_out) = terminal_out {
        log_outputs.push(terminal_out);
    }

    // Initialize log file output
    let espanso_dir = context::get_data_dir();
    let log_file_path = espanso_dir.join(LOG_FILE);
    let log_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(log_file_path)
        .expect("Cannot create log file.");
    let file_out = WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), log_file);
    log_outputs.push(file_out);

    CombinedLogger::init(
        log_outputs
    ).expect("Error opening log destination");

    // Activate logging for panics
    log_panics::init();

    info!("espanso version {}", VERSION);
    info!("starting daemon...");

    let (send_channel, receive_channel) = mpsc::channel();

    let context = context::new(send_channel.clone());

    let config_set_copy = config_set.clone();
    thread::Builder::new().name("daemon_background".to_string()).spawn(move || {
        daemon_background(receive_channel, config_set_copy);
    }).expect("Unable to spawn daemon background thread");

    let ipc_server = protocol::get_ipc_server(config_set, send_channel.clone());
    ipc_server.start();

    context.eventloop();
}

/// Background thread worker for the daemon
fn daemon_background(receive_channel: Receiver<Event>, config_set: ConfigSet) {
    let system_manager = system::get_manager();
    let config_manager = RuntimeConfigManager::new(config_set, system_manager);

    let ui_manager = ui::get_uimanager();
    ui_manager.notify("espanso is running!");

    let clipboard_manager = clipboard::get_manager();

    let keyboard_manager = keyboard::get_manager();

    let extensions = extension::get_extensions();

    let engine = Engine::new(&keyboard_manager,
                             &clipboard_manager,
                             &config_manager,
                             &ui_manager,
                             extensions,
    );

    let matcher = ScrollingMatcher::new(&config_manager, &engine);

    let event_manager = DefaultEventManager::new(
        receive_channel,
        vec!(&matcher),
        vec!(&engine, &matcher),
    );

    info!("espanso is running!");

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
            }else{
                println!("Error starting launchd daemon with status: {}", status);
            }
        }else{
            println!("Error starting launchd daemon: {}", res.unwrap_err());
        }
    }else{
        fork_daemon(config_set);
    }
}

#[cfg(target_os = "linux")]
fn start_daemon(config_set: ConfigSet) {
    if config_set.default.use_system_agent {
        // TODO: systemd
    }else{
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
        if pid > 0 {  // Parent process exit
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
    }else{
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

    let res = send_command(config_set, IPCCommand{
        id: "exit".to_owned(),
        payload: "".to_owned(),
    });

    if let Err(e) = res {
        println!("{}", e);
        exit(1);
    }else{
        exit(0);
    }
}

/// Kill the daemon if running and start it again
fn restart_main(config_set: ConfigSet) {
    // Kill the daemon if running
    let lock_file = acquire_lock();
    if lock_file.is_none() {
        // Terminate the current espanso daemon
        send_command(config_set.clone(), IPCCommand{
            id: "exit".to_owned(),
            payload: "".to_owned(),
        }).unwrap_or_else(|e| warn!("Unable to send IPC command to daemon: {}", e));
    }else{
        release_lock(lock_file.unwrap());
    }

    std::thread::sleep(Duration::from_millis(300));

    // Restart the daemon
    start_main(config_set);
}

/// Cli tool used to analyze active windows to extract useful information
/// to create configuration filters.
fn detect_main() {
    let system_manager = system::get_manager();

    println!("Listening for changes, now focus the window you want to analyze.");
    println!("You can terminate with CTRL+C\n");

    let mut last_title : String = "".to_owned();
    let mut last_class : String = "".to_owned();
    let mut last_exec : String = "".to_owned();

    loop {
        let curr_title = system_manager.get_current_window_title().unwrap_or_default();
        let curr_class = system_manager.get_current_window_class().unwrap_or_default();
        let curr_exec = system_manager.get_current_window_executable().unwrap_or_default();

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

/// Send the given command to the espanso daemon
fn cmd_main(config_set: ConfigSet, matches: &ArgMatches) {
    let command = if let Some(_) = matches.subcommand_matches("exit") {
        Some(IPCCommand {
            id: String::from("exit"),
            payload: String::from(""),
        })
    }else if let Some(_) = matches.subcommand_matches("toggle") {
        Some(IPCCommand {
            id: String::from("toggle"),
            payload: String::from(""),
        })
    }else if let Some(_) = matches.subcommand_matches("enable") {
        Some(IPCCommand {
            id: String::from("enable"),
            payload: String::from(""),
        })
    }else if let Some(_) = matches.subcommand_matches("disable") {
        Some(IPCCommand {
            id: String::from("disable"),
            payload: String::from(""),
        })
    }else{
        None
    };

    if let Some(command) = command {
        let res = send_command(config_set, command);

        if res.is_ok() {
            exit(0);
        }else{
            println!("{}", res.unwrap_err());
        }
    }

    exit(1);
}

fn send_command(config_set: ConfigSet, command: IPCCommand) -> Result<(), String> {
    let ipc_client = protocol::get_ipc_client(config_set);
    ipc_client.send_command(command)
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
    }else{
        println!("Error reading log file");
        exit(1);
    }
}

fn install_main(config_set: ConfigSet) {
    install::install(config_set);
}

fn uninstall_main(config_set: ConfigSet) {
    install::uninstall(config_set);
}

fn acquire_lock() -> Option<File> {
    let espanso_dir = context::get_data_dir();
    let lock_file_path = espanso_dir.join("espanso.lock");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(lock_file_path)
        .expect("Cannot create reference to lock file.");

    let res = file.try_lock_exclusive();

    if let Ok(_) = res {
        return Some(file)
    }

    None
}

fn release_lock(lock_file: File) {
    lock_file.unlock().unwrap()
}

/// Used to make sure all the required dependencies are present before starting espanso.
fn precheck_guard() {
    let satisfied = check::check_dependencies();
    if !satisfied {
        println!();
        println!("Pre-check was not successful, espanso could not be started.");
        exit(5);
    }
}
use std::thread;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process::exit;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use clap::{App, Arg, SubCommand};
use fs2::FileExt;
use log::{error, info, LevelFilter};
use simplelog::{CombinedLogger, SharedLogger, TerminalMode, TermLogger};

use crate::config::ConfigSet;
use crate::config::runtime::RuntimeConfigManager;
use crate::engine::Engine;
use crate::event::*;
use crate::event::manager::{DefaultEventManager, EventManager};
use crate::matcher::scrolling::ScrollingMatcher;
use crate::system::SystemManager;
use crate::ui::UIManager;

mod ui;
mod event;
mod bridge;
mod engine;
mod config;
mod system;
mod context;
mod matcher;
mod keyboard;
mod clipboard;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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
        .subcommand(SubCommand::with_name("dump")
            .about("Prints all current configuration options."))
        .subcommand(SubCommand::with_name("detect")
            .about("Tool to detect current window properties, to simplify filters creation."))
        .subcommand(SubCommand::with_name("daemon")
            .about("Start the daemon without spawning a new process."))
        .subcommand(SubCommand::with_name("start")
            .about("Start the daemon spawning a new process in the background."))
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

    if let Some(matches) = matches.subcommand_matches("dump") {
        println!("{:#?}", config_set);
        return;
    }

    if let Some(matches) = matches.subcommand_matches("detect") {
        detect_main();
        return;
    }

    if let Some(matches) = matches.subcommand_matches("daemon") {
        daemon_main(config_set);
        return;
    }

    if let Some(matches) = matches.subcommand_matches("start") {
        start_main(config_set);
        return;
    }

    if let Some(matches) = matches.subcommand_matches("status") {
        status_main();
        return;
    }
}

/// Daemon subcommand, start the event loop and spawn a background thread worker
fn daemon_main(config_set: ConfigSet) {
    // Try to acquire lock file
    let lock_file = acquire_lock();
    if lock_file.is_none() {
        println!("espanso is already running.");
        exit(3);
    }

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

    //TODO: WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_binary.log").unwrap()),
    CombinedLogger::init(
        log_outputs
    ).expect("Error opening log destination");

    info!("starting daemon...");

    let (send_channel, receive_channel) = mpsc::channel();

    let context = context::new(send_channel);

    thread::spawn(move || {
        daemon_background(receive_channel, config_set);
    });

    context.eventloop();
}

/// Background thread worker for the daemon
fn daemon_background(receive_channel: Receiver<Event>, config_set: ConfigSet) {
    let system_manager = system::get_manager();
    let config_manager = RuntimeConfigManager::new(config_set, system_manager);

    let ui_manager = ui::get_uimanager();
    ui_manager.notify("espanso is running!");

    let clipboard_manager = clipboard::get_manager();

    let manager = keyboard::get_manager();

    let engine = Engine::new(&manager,
                             &clipboard_manager,
                             &config_manager,
                             &ui_manager
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

    if cfg!(target_os = "windows") {
        // TODO: start windows detached
    }else{
        unsafe {
            let pid = libc::fork();
            if pid < 0 {
                error!("Unable to fork.");
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
        }

        daemon_main(config_set);
    }
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
            println!("");
        }

        last_title = curr_title;
        last_class = curr_class;
        last_exec = curr_exec;

        thread::sleep(Duration::from_millis(500));
    }
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
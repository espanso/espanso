use std::sync::{mpsc};
use crate::matcher::scrolling::ScrollingMatcher;
use crate::engine::Engine;
use crate::config::ConfigSet;
use crate::config::runtime::RuntimeConfigManager;
use crate::system::SystemManager;
use crate::ui::UIManager;
use crate::event::*;
use crate::event::manager::{EventManager, DefaultEventManager};
use std::{thread};
use clap::{App, Arg, SubCommand};
use std::path::Path;
use std::sync::mpsc::Receiver;
use log::{info, error, LevelFilter};
use simplelog::{CombinedLogger, TermLogger, TerminalMode, SharedLogger};
use std::process::exit;
use std::time::Duration;

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
        .get_matches();


    // Setup logging
    let log_level = match matches.occurrences_of("v") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 | _ => LevelFilter::Debug,
    };
    let mut log_outputs: Vec<Box<dyn SharedLogger>> = Vec::new();

    // Initialize terminal output
    let terminal_out = TermLogger::new(log_level, simplelog::Config::default(), TerminalMode::Mixed);
    if let Some(terminal_out) = terminal_out {
        log_outputs.push(terminal_out);
    }

    //TODO: WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_binary.log").unwrap()),
    CombinedLogger::init(
        log_outputs
    ).expect("Error opening log destination");

    info!("espanso is starting...");

    let config_set = match matches.value_of("config") {
        None => {
            info!("loading configuration from default location...");
            ConfigSet::load_default()
        },
        Some(path) => {
            info!("loading configuration from custom location: {}", path);
            ConfigSet::load(Path::new(path))
        },
    }.unwrap_or_else(|e| {
        error!("{}", e);
        exit(1);
    });

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
}

fn daemon_main(config_set: ConfigSet) {
    info!("starting daemon...");

    let (send_channel, receive_channel) = mpsc::channel();

    let context = context::new(send_channel);

    thread::spawn(move || {
        daemon_background(receive_channel, config_set);
    });

    context.eventloop();
}

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
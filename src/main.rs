#[macro_use]
extern crate num_derive;

use std::sync::{mpsc, Arc};
use crate::matcher::Matcher;
use crate::matcher::scrolling::ScrollingMatcher;
use crate::engine::Engine;
use crate::clipboard::ClipboardManager;
use crate::config::ConfigSet;
use crate::config::runtime::RuntimeConfigManager;
use crate::ui::UIManager;
use crate::context::Context;
use crate::event::*;
use crate::event::manager::{EventManager, DefaultEventManager};
use std::{thread, time};
use clap::{App, Arg};
use std::path::Path;
use std::sync::mpsc::Receiver;
use log::{info, error, LevelFilter};
use simplelog::{CombinedLogger, TermLogger, TerminalMode};
use std::process::exit;

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
        .arg(Arg::with_name("dump")
            .long("dump")
            .help("Prints all current configuration options."))
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .get_matches();


    // Setup logging
    let log_level = match matches.occurrences_of("v") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 | _ => LevelFilter::Debug,
    };
    CombinedLogger::init(
        vec![
            TermLogger::new(log_level, simplelog::Config::default(), TerminalMode::Mixed).unwrap(),
            //WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_binary.log").unwrap()),
        ]
    ).unwrap();

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

    if matches.is_present("dump") {
        println!("{:#?}", config_set);
        return;
    }

    espanso_main(config_set);
}

fn espanso_main(config_set: ConfigSet) {
    let (send_channel, receive_channel) = mpsc::channel();

    let context = context::new(send_channel);

    thread::spawn(move || {
        espanso_background(receive_channel, config_set);
    });

    context.eventloop();
}

fn espanso_background(receive_channel: Receiver<Event>, config_set: ConfigSet) {
    let system_manager = system::get_manager();
    let config_manager = RuntimeConfigManager::new(config_set, system_manager);

    let ui_manager = ui::get_uimanager();
    ui_manager.notify("espanso is running!");

    let clipboard_manager = clipboard::get_manager();

    let sender = keyboard::get_sender();  // TODO: rename manager

    // TODO: change sender to move to reference
    let engine = Engine::new(sender,
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

    event_manager.eventloop();
}
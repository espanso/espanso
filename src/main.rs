use std::sync::{mpsc};
use crate::keyboard::{KeyboardInterceptor, KeyEvent};
use crate::matcher::Matcher;
use crate::matcher::scrolling::ScrollingMatcher;
use crate::engine::Engine;
use crate::config::{ConfigSet, RuntimeConfigManager};
use crate::ui::UIManager;
use std::thread;
use clap::{App, Arg};
use std::path::Path;
use std::sync::mpsc::Receiver;

mod ui;
mod bridge;
mod engine;
mod config;
mod system;
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

    let config_set = match matches.value_of("config") {
        None => {ConfigSet::load_default()},
        Some(path) => {ConfigSet::load(Path::new(path))},
    };

    if matches.is_present("dump") {
        println!("{:#?}", config_set);
        return;
    }

    espanso_main(config_set);
}

fn espanso_main(config_set: ConfigSet) {
    let (txc, rxc) = mpsc::channel();

    thread::spawn(move || {
        espanso_background(rxc, config_set);
    });

    let interceptor = keyboard::get_interceptor(txc);
    interceptor.initialize();
    interceptor.start();
}

fn espanso_background(rxc: Receiver<KeyEvent>, config_set: ConfigSet) {
    let system_manager = system::get_manager();
    let config_manager = RuntimeConfigManager::new(config_set, system_manager);

    let ui_manager = ui::get_uimanager();
    ui_manager.notify("Hello guys");

    let clipboard_manager = clipboard::get_manager();

    let sender = keyboard::get_sender();

    let engine = Engine::new(sender,
                             &clipboard_manager,
                             &config_manager);

    let matcher = ScrollingMatcher::new(&config_manager, engine);
    matcher.watch(rxc);
}
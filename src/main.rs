use std::sync::mpsc;
use crate::keyboard::KeyboardInterceptor;
use crate::matcher::Matcher;
use crate::matcher::scrolling::ScrollingMatcher;
use crate::engine::Engine;
use crate::config::Configs;
use std::thread;
use clap::{App, Arg};
use std::path::Path;

mod keyboard;
mod matcher;
mod engine;
mod config;
mod ui;

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
            .help("Sets a custom config file. If not specified, reads the default $HOME/.espanso file, creating it if not present.")
            .takes_value(true))
        .arg(Arg::with_name("dump")
            .long("dump")
            .help("Prints all current configuration options."))
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .get_matches();

    let configs = match matches.value_of("config") {
        None => {Configs::load_default()},
        Some(path) => {Configs::load(Path::new(path))},
    };

    if matches.is_present("dump") {
        println!("{:#?}", configs);
        return;
    }

    espanso_main(configs);
}

fn espanso_main(configs: Configs) {
    let ui_manager = UIManager::new();

    let (txc, rxc) = mpsc::channel();

    let sender = keyboard::get_sender();

    let engine = Engine::new(sender, configs.clone());

    thread::spawn(move || {
        let matcher = ScrollingMatcher::new(configs.clone(), engine);
        matcher.watch(rxc);
    });

    let interceptor = keyboard::get_interceptor(txc);
    interceptor.initialize();
    interceptor.start();
}
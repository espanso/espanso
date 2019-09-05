use std::sync::mpsc;
use crate::keyboard::KeyboardInterceptor;
use crate::matcher::Matcher;
use crate::matcher::scrolling::ScrollingMatcher;
use crate::engine::Engine;
use crate::config::Configs;
use std::thread;

mod keyboard;
mod matcher;
mod engine;
mod config;

fn main() {
    let configs = Configs::load_default();
    println!("{:#?}", configs);

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
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

    let (txc, rxc) = mpsc::channel();

    let sender = keyboard::get_sender();

    let engine = Engine::new(sender);

    thread::spawn(move || {
        let matcher = ScrollingMatcher::new(configs.matches.to_vec(), engine);
        matcher.watch(rxc);
    });

    let interceptor = keyboard::get_interceptor(txc);
    interceptor.initialize();
    interceptor.start();
}
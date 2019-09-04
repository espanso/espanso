use std::sync::mpsc;
use crate::keyboard::KeyboardInterceptor;
use crate::matcher::Matcher;
use crate::matcher::Match;
use crate::matcher::scrolling::ScrollingMatcher;
use crate::engine::Engine;
use crate::config::Configs;
use std::path::Path;

mod keyboard;
mod matcher;
mod engine;
mod config;

fn main() {
    let configs = Configs::load_default();

    let (txc, rxc) = mpsc::channel();

    let interceptor = keyboard::get_interceptor(txc);
    interceptor.initialize();
    interceptor.start();

    let sender = keyboard::get_sender();

    let engine = Engine::new(&sender);

    println!("espanso is running!");

    let mut matcher = ScrollingMatcher::new(&configs.matches, &engine);
    matcher.watch(&rxc);
}
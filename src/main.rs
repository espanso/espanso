use std::sync::mpsc;
use crate::keyboard::KeyboardInterceptor;
use crate::matcher::Matcher;
use crate::matcher::scrolling::ScrollingMatcher;
use crate::engine::Engine;

mod keyboard;
mod matcher;
mod engine;

fn main() {
    println!("espanso is running!");

    let (txc, rxc) = mpsc::channel();

    let interceptor = keyboard::get_interceptor(txc);
    interceptor.initialize();
    interceptor.start();

    let sender = keyboard::get_sender();

    let engine = Engine::new(&sender);

    let matcher = ScrollingMatcher::new(&engine);
    matcher.watch(&rxc);
}
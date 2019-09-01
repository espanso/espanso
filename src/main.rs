use std::sync::mpsc;
use crate::keyboard::KeyboardInterceptor;
use crate::matcher::Matcher;
use crate::matcher::Match;
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

    let matches = vec![Match{trigger:"e'".to_owned(), result: "è".to_owned()},
                       Match{trigger:":lol".to_owned(), result: "😂".to_owned()},
                       Match{trigger:":lll".to_owned(), result: "hello".to_owned()},
    ];

    let mut matcher = ScrollingMatcher::new(&matches, &engine);
    matcher.watch(&rxc);
}
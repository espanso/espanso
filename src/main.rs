use std::thread::sleep;
use std::time::Duration;
use std::sync::mpsc;
use crate::keyboard::KeyboardInterceptor;
use crate::keyboard::KeyboardSender;
use crate::matcher::Matcher;

mod keyboard;
mod matcher;

fn main() {
    println!("Hello, world from Rust!");

    let (sender, receiver) = mpsc::channel();

    let (interceptor, sender) = keyboard::get_backend(sender);
    interceptor.initialize();
    interceptor.start();

    let matcher = Matcher{receiver};
    matcher.watch();
}
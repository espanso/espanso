use std::thread::sleep;
use std::time::Duration;
use crate::keyboard::KeyboardBackend;
use std::sync::mpsc;

mod keyboard;

fn main() {
    println!("Hello, world from Rust!");

    let (sender, receiver) = mpsc::channel();

    let keyboard = keyboard::get_backend(sender);
    keyboard.initialize();
    keyboard.start();

    loop {
        match receiver.recv() {
            Ok(c) => {
                println!("Yeah {}",c);
            },
            Err(_) => panic!("Worker threads disconnected before the solution was found!"),
        }
    }
}
#[link(name="winbridge", kind="static")]
extern {
    fn initialize();
    fn eventloop();
}

fn main() {
    println!("Hello, world from Rust!");

    // calling the function from foo library
    unsafe {
        initialize();
        eventloop();
    };
}
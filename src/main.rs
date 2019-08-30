extern fn keypress_callback(raw_buffer: *const i32, len: i32) {
    unsafe {
        let buffer = std::slice::from_raw_parts(raw_buffer, len as usize);
        println!("{}", std::char::from_u32(buffer[0] as u32).unwrap());
    }
}

#[link(name="winbridge", kind="static")]
extern {
    fn initialize();
    fn eventloop();
    fn register_keypress_callback(cb: extern fn(*const i32, i32));
}

fn main() {
    println!("Hello, world from Rust!");

    // calling the function from foo library
    unsafe {
        initialize();

        register_keypress_callback(keypress_callback);

        eventloop();
    };
}
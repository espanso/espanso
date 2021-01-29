fn main() {
  println!("Hello, world!z");

  let source = espanso_detect::win32::Win32Source::new(Box::new(|event| {
      println!("ev {:?}", event);
  }));
  source.eventloop();
}

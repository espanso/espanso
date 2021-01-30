use espanso_detect::event::{InputEvent, Status};

fn main() {
  println!("Hello, world!z");

  let icon_paths = vec![
    (
      espanso_ui::icons::TrayIcon::Normal,
      r"C:\Users\Freddy\AppData\Local\espanso\espanso.ico".to_string(),
    ),
    (
      espanso_ui::icons::TrayIcon::Disabled,
      r"C:\Users\Freddy\AppData\Local\espanso\espansored.ico".to_string(),
    ),
  ];

  let (remote, mut eventloop) = espanso_ui::win32::create(espanso_ui::win32::Win32UIOptions {
    show_icon: true,
    icon_paths: &icon_paths,
    notification_icon_path: r"C:\Users\Freddy\Insync\Development\Espanso\Images\icongreensmall.png".to_string(),
  });

  std::thread::spawn(move || {
    let mut source = espanso_detect::win32::Win32Source::new();
    source.initialize();
    source.eventloop(Box::new(move |event: InputEvent| {
      println!("ev {:?}", event);
      match event {
        InputEvent::Mouse(_) => {}
        InputEvent::Keyboard(evt) => {
          if evt.key == espanso_detect::event::Key::Shift && evt.status == Status::Pressed {
            //remote.update_tray_icon(espanso_ui::icons::TrayIcon::Disabled);
            remote.show_notification("Espanso is running!");
          }
        }
      }
    }));
  });

  eventloop.initialize();
  eventloop.run(Box::new(|event| {
    println!("ui {:?}", event);
  }))
}

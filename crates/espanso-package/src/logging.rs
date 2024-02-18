#[macro_export]
macro_rules! info_println {
  ($($tts:tt)*) => {
    println!($($tts)*);
    log::info!($($tts)*);
  }
}

#[macro_export]
macro_rules! warn_eprintln {
  ($($tts:tt)*) => {
    eprintln!($($tts)*);
    log::warn!($($tts)*);
  }
}

#[macro_export]
macro_rules! error_eprintln {
  ($($tts:tt)*) => {
    eprintln!($($tts)*);
    log::error!($($tts)*);
  }
}

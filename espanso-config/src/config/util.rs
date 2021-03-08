#[macro_export]
macro_rules! merge {
  ( $t:ident, $child:expr, $parent:expr, $( $x:ident ),* ) => {
    {
      $(
        if $child.$x.is_none() {
          $child.$x = $parent.$x.clone();
        }
      )*
      
      // Build a temporary object to verify that all fields
      // are being used at compile time
      $t {
        $(
          $x: None,
        )*
      };
    }
  };
}

pub fn os_matches(os: &str) -> bool {
  match os {
    "macos" => cfg!(target_os = "macos"),
    "windows" => cfg!(target_os = "windows"),
    "linux" => cfg!(target_os = "linux"),
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[cfg(target_os = "linux")]
  fn os_matches_linux() {
    assert!(os_matches("linux"));
    assert!(!os_matches("windows"));
    assert!(!os_matches("macos"));
    assert!(!os_matches("invalid"));
  }


  #[test]
  #[cfg(target_os = "macos")]
  fn os_matches_macos() {
    assert!(os_matches("macos"));
    assert!(!os_matches("windows"));
    assert!(!os_matches("linux"));
    assert!(!os_matches("invalid"));
  }

  #[test]
  #[cfg(target_os = "windows")]
  fn os_matches_windows() {
    assert!(os_matches("windows"));
    assert!(!os_matches("macos"));
    assert!(!os_matches("linux"));
    assert!(!os_matches("invalid"));
  }
}
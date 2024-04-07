//! ```cargo
//! [dependencies]
//! envmnt = "*"
//! ```

use std::process::Command;

#[derive(Debug, PartialEq)]
enum Profile {
  Debug,
  Release,
}

fn main() {
  let profile = if envmnt::get_or_panic("RELEASE") == "true" {
    Profile::Release
  } else {
    Profile::Debug
  };

  println!("Using profile: {:?}", profile);

  let wayland = envmnt::get_or("NO_X11", "false") == "true";
  if wayland {
    println!("Using Wayland feature");
  }
  else {
    println!("Using X11 default feature");
  }

  let mut args = Vec::new();

  args.push("test");
  args.push("--workspace");
  args.push("--exclude");
  args.push("espanso-modulo");
  args.push("--exclude");
  args.push("espanso-ipc");

  if profile == Profile::Release {
    args.push("--release");
  }

  let override_target_arch = envmnt::get_or("BUILD_ARCH", "current");
  if override_target_arch != "current" {
    args.push("--target");
    args.push(&override_target_arch);
  }

  let mut features = Vec::new();
  if wayland {
    features.push("wayland");
  }
  // On linux, we don't want to rely on OpenSSL to avoid dependency issues
  // https://github.com/espanso/espanso/issues/1056
  if cfg!(target_os = "linux") {
    features.push("vendored-tls")
  } else {
    features.push("native-tls")
  }

  let features_flag = features.join(" ");

  args.push("--no-default-features");
  args.push("--features");
  args.push(&features_flag);

  println!("Calling with args: {:?}", args);

  let mut cmd = Command::new("cargo");
  cmd.args(&args);

  // Remove cargo/rust-specific env variables, as otherwise they mess up the
  // nested cargo build call.
  let all_vars = envmnt::vars();
  for (key, _) in all_vars {
    if key.starts_with("CARGO") || (key.starts_with("RUST") && !key.starts_with("RUSTUP")) {
      //println!("Removing {}", key);
      cmd.env_remove(key);
    }
  }

  let mut handle = cmd.spawn().expect("cargo test failed");
  let result = handle.wait().expect("unable to read cargo exit status");
  if !result.success() {
    panic!("cargo test failed");
  }
}

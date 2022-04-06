/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
 *
 * espanso is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * espanso is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with espanso.  If not, see <https://www.gnu.org/licenses/>.
 */

#[cfg(target_os = "windows")]
fn cc_config() {
  println!("cargo:rerun-if-changed=src/win32/native.cpp");
  println!("cargo:rerun-if-changed=src/win32/native.h");
  cc::Build::new()
    .cpp(true)
    .include("src/win32/native.h")
    .file("src/win32/native.cpp")
    .compile("espansoinfo");

  println!("cargo:rustc-link-lib=static=espansoinfo");
  println!("cargo:rustc-link-lib=dylib=user32");
  #[cfg(target_env = "gnu")]
  println!("cargo:rustc-link-lib=dylib=stdc++");
}

#[cfg(target_os = "linux")]
fn cc_config() {
  if cfg!(not(feature = "wayland")) {
    println!("cargo:rerun-if-changed=src/x11/native.h");
    println!("cargo:rerun-if-changed=src/x11/native.c");
    cc::Build::new()
      .cpp(true)
      .include("src/x11")
      .file("src/x11/native.cpp")
      .compile("espansoinfo");

    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu/");
    println!("cargo:rustc-link-lib=static=espansoinfo");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=X11");
  } else {
    println!("cargo:rerun-if-changed=src/wayland/native.h");
    println!("cargo:rerun-if-changed=src/wayland/native.c");
    cc::Build::new()
      .cpp(true)
      .include("src/wayland")
      .include("/usr/include/dbus-1.0")
      .include("/usr/lib64/dbus-1.0/include")
      .file("src/wayland/native.cpp")
      .compile("espansoinfo");

    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu/");
    println!("cargo:rustc-link-lib=static=espansoinfo");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=dbus-1");
  }
}

#[cfg(target_os = "macos")]
fn cc_config() {
  println!("cargo:rerun-if-changed=src/cocoa/native.mm");
  println!("cargo:rerun-if-changed=src/cocoa/native.h");
  cc::Build::new()
    .cpp(true)
    .include("src/cocoa/native.h")
    .file("src/cocoa/native.mm")
    .compile("espansoinfo");
  println!("cargo:rustc-link-lib=dylib=c++");
  println!("cargo:rustc-link-lib=static=espansoinfo");
  println!("cargo:rustc-link-lib=framework=Cocoa");
}

fn main() {
  cc_config();
}

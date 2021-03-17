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
    .compile("espansoclipboard");

  println!("cargo:rustc-link-lib=static=espansoclipboard");
  println!("cargo:rustc-link-lib=dylib=user32");
  #[cfg(target_env = "gnu")]
  println!("cargo:rustc-link-lib=dylib=stdc++");
}

#[cfg(target_os = "linux")]
fn cc_config() {
  if cfg!(not(feature = "wayland")) {
    println!("cargo:rerun-if-changed=src/x11/native/native.h");
    println!("cargo:rerun-if-changed=src/x11/native/native.c");
    cc::Build::new()
      .cpp(true)
      .include("src/x11/native/clip/clip.h")
      .include("src/x11/native/clip/clip_common.h")
      .include("src/x11/native/clip/clip_lock_impl.h")
      .include("src/x11/native/clip/clip_x11_png.h")
      .include("src/x11/native/native.h")
      .file("src/x11/native/clip/clip.cpp")
      .file("src/x11/native/clip/clip_x11.cpp")
      .file("src/x11/native/clip/image.cpp")
      .file("src/x11/native/native.cpp")
      .compile("espansoclipboardx11");

    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu/");
    println!("cargo:rustc-link-lib=static=espansoclipboardx11");
    println!("cargo:rustc-link-lib=dylib=xcb");
  } else {
    // Nothing to compile on wayland
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
    .compile("espansoclipboard");
  println!("cargo:rustc-link-lib=dylib=c++");
  println!("cargo:rustc-link-lib=static=espansoclipboard");
  println!("cargo:rustc-link-lib=framework=Cocoa");
}

fn main() {
  cc_config();
}

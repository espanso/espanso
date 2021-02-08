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
    .include("src/win32/WinToast/wintoastlib.h")
    .include("src/win32/json/json.hpp")
    .file("src/win32/native.cpp")
    .file("src/win32/WinToast/wintoastlib.cpp")
    .compile("espansoui");

  println!("cargo:rustc-link-lib=static=espansoui");
  println!("cargo:rustc-link-lib=dylib=user32");
  #[cfg(target_env = "gnu")]
  println!("cargo:rustc-link-lib=dylib=stdc++");
  #[cfg(target_env = "gnu")]
  println!("cargo:rustc-link-lib=dylib=gdiplus");
}

#[cfg(target_os = "linux")]
fn cc_config() {
  // Nothing to link on linux
}

#[cfg(target_os = "macos")]
fn cc_config() {
  println!("cargo:rerun-if-changed=src/mac/native.mm");
  println!("cargo:rerun-if-changed=src/mac/native.h");
  println!("cargo:rerun-if-changed=src/mac/AppDelegate.mm");
  println!("cargo:rerun-if-changed=src/mac/AppDelegate.h");
  cc::Build::new()
    .cpp(true)
    .include("src/mac/native.h")
    .include("src/mac/AppDelegate.h")
    .file("src/mac/native.mm")
    .file("src/mac/AppDelegate.mm")
    .compile("espansoui");
  println!("cargo:rustc-link-lib=dylib=c++");
  println!("cargo:rustc-link-lib=static=espansoui");
  println!("cargo:rustc-link-lib=framework=Cocoa");
  println!("cargo:rustc-link-lib=framework=IOKit");
}

fn main() {
  cc_config();
}

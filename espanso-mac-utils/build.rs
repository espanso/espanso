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

#[cfg(not(target_os = "macos"))]
fn cc_config() {
  // Do nothing on Linux and Windows
}

#[cfg(target_os = "macos")]
fn cc_config() {
  println!("cargo:rerun-if-changed=src/native.mm");
  println!("cargo:rerun-if-changed=src/native.h");
  cc::Build::new()
    .cpp(true)
    .include("src/native.h")
    .file("src/native.mm")
    .compile("espansomacutils");
  println!("cargo:rustc-link-lib=dylib=c++");
  println!("cargo:rustc-link-lib=static=espansomacutils");
  println!("cargo:rustc-link-lib=framework=Cocoa");
}

fn main() {
  cc_config();
}

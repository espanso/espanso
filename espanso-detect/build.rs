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
        .compile("espansodetect");

    println!("cargo:rustc-link-lib=static=espansodetect");
    println!("cargo:rustc-link-lib=dylib=user32");
    #[cfg(target_env = "gnu")]
    println!("cargo:rustc-link-lib=dylib=stdc++");
}

#[cfg(target_os = "linux")]
fn cc_config() {
    println!("cargo:rerun-if-changed=src/x11/native.cpp");
    println!("cargo:rerun-if-changed=src/x11/native.h");
    println!("cargo:rerun-if-changed=src/evdev/native.cpp");
    println!("cargo:rerun-if-changed=src/evdev/native.h");

    if cfg!(not(feature = "wayland")) {
        cc::Build::new()
            .cpp(true)
            .include("src/x11")
            .file("src/x11/native.cpp")
            .compile("espansodetect");

        println!("cargo:rustc-link-lib=static=espansodetect");
        println!("cargo:rustc-link-lib=dylib=X11");
        println!("cargo:rustc-link-lib=dylib=Xtst");
    }

    cc::Build::new()
        .cpp(true)
        .include("src/evdev")
        .file("src/evdev/native.cpp")
        .compile("espansodetectevdev");

    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu/");
    println!("cargo:rustc-link-lib=static=espansodetectevdev");
    println!("cargo:rustc-link-lib=dylib=xkbcommon");
}

#[cfg(target_os = "macos")]
fn cc_config() {
    println!("cargo:rerun-if-changed=src/mac/native.mm");
    println!("cargo:rerun-if-changed=src/mac/native.h");
    cc::Build::new()
        .cpp(true)
        .include("src/mac/native.h")
        .file("src/mac/native.mm")
        .compile("espansodetect");
    println!("cargo:rustc-link-lib=dylib=c++");
    println!("cargo:rustc-link-lib=static=espansodetect");
    println!("cargo:rustc-link-lib=framework=Cocoa");
    println!("cargo:rustc-link-lib=framework=Carbon");
}

fn main() {
    cc_config();
}

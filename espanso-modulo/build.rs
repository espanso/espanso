/*
 * This file is part of espanso.
 *
 * Copyright (C) 2021 Federico Terzi
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

use std::path::PathBuf;

#[cfg(not(target_os = "windows"))]
use std::path::Path;

#[cfg(not(target_os = "linux"))]
const WX_WIDGETS_ARCHIVE_NAME: &str = "wxWidgets-3.1.5.zip";

#[cfg(not(target_os = "linux"))]
const WX_WIDGETS_BUILD_OUT_DIR_ENV_NAME: &str = "WX_WIDGETS_BUILD_OUT_DIR";

#[cfg(target_os = "windows")]
fn build_native() {
  use std::process::Command;

  let project_dir =
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
  let wx_archive = project_dir.join("vendor").join(WX_WIDGETS_ARCHIVE_NAME);
  assert!(wx_archive.is_file(), "could not find wxWidgets archive!");

  let out_dir = if let Ok(out_path) = std::env::var(WX_WIDGETS_BUILD_OUT_DIR_ENV_NAME) {
    println!("detected wxWidgets build output directory override: {out_path}");
    let path = PathBuf::from(out_path);
    std::fs::create_dir_all(&path).expect("unable to create wxWidgets out dir");
    path
  } else {
    PathBuf::from(std::env::var("OUT_DIR").expect("missing OUT_DIR"))
  };
  let out_wx_dir = out_dir.join("wx");

  if !out_wx_dir.is_dir() {
    // Extract the wxWidgets archive
    let wx_archive =
      std::fs::File::open(&wx_archive).expect("unable to open wxWidgets source archive");
    let mut archive = zip::ZipArchive::new(wx_archive).expect("unable to read wxWidgets archive");
    archive
      .extract(&out_wx_dir)
      .expect("unable to extract wxWidgets source dir");

    // Compile wxWidgets
    let tool = cc::windows_registry::find_tool("msvc", "devenv")
      .expect("unable to locate MSVC compiler, did you install Visual Studio?");
    let mut vcvars_path = None;
    let mut current_root = tool.path();
    while let Some(parent) = current_root.parent() {
      let target = parent
        .join("VC")
        .join("Auxiliary")
        .join("Build")
        .join("vcvars64.bat");
      if target.exists() {
        vcvars_path = Some(target);
        break;
      }
      current_root = parent;
    }

    let vcvars_path = vcvars_path.expect("unable to find vcvars64.bat file");
    let mut handle = Command::new("cmd")
      .current_dir(
        out_wx_dir
          .join("build")
          .join("msw")
          .to_string_lossy()
          .to_string(),
      )
      .args([
        "/k",
        &vcvars_path.to_string_lossy(),
        "&",
        "nmake",
        "/f",
        "makefile.vc",
        "BUILD=release",
        "TARGET_CPU=X64",
        "&",
        "exit",
      ])
      .spawn()
      .expect("failed to execute nmake");
    if !handle
      .wait()
      .expect("unable to wait for nmake command")
      .success()
    {
      panic!("nmake returned non-zero exit code!");
    }
  }

  // Make sure wxWidgets is compiled
  if !out_wx_dir
    .join("build")
    .join("msw")
    .join("vc_mswu_x64")
    .is_dir()
  {
    panic!("wxWidgets is not compiled correctly, missing 'build/msw/vc_mswu_x64' directory")
  }

  let wx_include_dir = out_wx_dir.join("include");
  let wx_include_msvc_dir = wx_include_dir.join("msvc");
  let wx_lib_dir = out_wx_dir.join("lib").join("vc_x64_lib");

  cc::Build::new()
    .cpp(true)
    .file("src/sys/form/form.cpp")
    .file("src/sys/search/search.cpp")
    .file("src/sys/common/common.cpp")
    .file("src/sys/wizard/wizard.cpp")
    .file("src/sys/wizard/wizard_gui.cpp")
    .file("src/sys/welcome/welcome.cpp")
    .file("src/sys/welcome/welcome_gui.cpp")
    .file("src/sys/textview/textview.cpp")
    .file("src/sys/textview/textview_gui.cpp")
    .file("src/sys/troubleshooting/troubleshooting.cpp")
    .file("src/sys/troubleshooting/troubleshooting_gui.cpp")
    .flag("/EHsc")
    .include(wx_include_dir)
    .include(wx_include_msvc_dir)
    .compile("espansomodulosys");

  // Add resources (manifest)
  let mut resources = winres::WindowsResource::new();
  resources.set_manifest(include_str!("res/win.manifest"));
  resources
    .compile()
    .expect("unable to compile resource file");

  println!(
    "cargo:rustc-link-search=native={}",
    wx_lib_dir.to_string_lossy()
  );
}

#[cfg(target_os = "macos")]
fn build_native() {
  use std::process::Command;

  let project_dir =
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
  let wx_archive = project_dir.join("vendor").join(WX_WIDGETS_ARCHIVE_NAME);
  assert!(wx_archive.is_file(), "could not find wxWidgets archive!");

  let out_dir = if let Ok(out_path) = std::env::var(WX_WIDGETS_BUILD_OUT_DIR_ENV_NAME) {
    println!("detected wxWidgets build output directory override: {out_path}");
    let path = PathBuf::from(out_path);
    std::fs::create_dir_all(&path).expect("unable to create wxWidgets out dir");
    path
  } else {
    PathBuf::from(std::env::var("OUT_DIR").expect("missing OUT_DIR"))
  };
  let out_wx_dir = out_dir.join("wx");
  println!("wxWidgets will be compiled into: {}", out_wx_dir.display());

  let target_arch = match std::env::var("CARGO_CFG_TARGET_ARCH")
    .expect("unable to read target arch")
    .as_str()
  {
    "x86_64" => "x86_64",
    "aarch64" => "arm64",
    arch => panic!("unsupported arch {arch}"),
  };

  let should_use_ci_m1_workaround =
    std::env::var("CI").unwrap_or_default() == "true" && target_arch == "arm64";

  if !out_wx_dir.is_dir() {
    // Extract the wxWidgets archive
    let wx_archive =
      std::fs::File::open(&wx_archive).expect("unable to open wxWidgets source archive");
    let mut archive = zip::ZipArchive::new(wx_archive).expect("unable to read wxWidgets archive");
    archive
      .extract(&out_wx_dir)
      .expect("unable to extract wxWidgets source dir");

    // Compile wxWidgets
    let build_dir = out_wx_dir.join("build-cocoa");
    std::fs::create_dir_all(&build_dir).expect("unable to create build-cocoa directory");

    let mut handle = if should_use_ci_m1_workaround {
      // Because of a configuration problem on the GitHub CI pipeline,
      // we need to use a series of workarounds to build for M1 machines.
      // See: https://github.com/actions/virtual-environments/issues/3288#issuecomment-830207746
      Command::new(out_wx_dir.join("configure"))
        .current_dir(build_dir.to_string_lossy().to_string())
        .args([
          "--disable-shared",
          "--without-libtiff",
          "--without-liblzma",
          "--with-libjpeg=builtin",
          "--with-libpng=builtin",
          "--enable-universal-binary=arm64,x86_64",
        ])
        .spawn()
        .expect("failed to execute configure")
    } else {
      Command::new(out_wx_dir.join("configure"))
        .current_dir(build_dir.to_string_lossy().to_string())
        .args([
          "--disable-shared",
          "--without-libtiff",
          "--without-liblzma",
          "--with-libjpeg=builtin",
          "--with-libpng=builtin",
          &format!("--enable-macosx_arch={target_arch}"),
        ])
        .spawn()
        .expect("failed to execute configure")
    };

    if !handle
      .wait()
      .expect("unable to wait for configure command")
      .success()
    {
      panic!("configure returned non-zero exit code!");
    }

    let mut handle = Command::new("make")
      .current_dir(build_dir.to_string_lossy().to_string())
      .args(["-j8"])
      .spawn()
      .expect("failed to execute make");
    if !handle
      .wait()
      .expect("unable to wait for make command")
      .success()
    {
      panic!("make returned non-zero exit code!");
    }
  }

  // Make sure wxWidgets is compiled
  assert!(
    out_wx_dir.join("build-cocoa").is_dir(),
    "wxWidgets is not compiled correctly, missing 'build-cocoa/' directory"
  );

  // If using the M1 CI workaround, convert all the universal libraries to arm64 ones
  // This is needed until https://github.com/rust-lang/rust/issues/55235 is fixed
  if should_use_ci_m1_workaround {
    convert_fat_libraries_to_arm(&out_wx_dir.join("build-cocoa").join("lib"));
    convert_fat_libraries_to_arm(&out_wx_dir.join("build-cocoa"));
  }

  let config_path = out_wx_dir.join("build-cocoa").join("wx-config");
  let cpp_flags = get_cpp_flags(&config_path);

  let mut build = cc::Build::new();
  build
    .cpp(true)
    .file("src/sys/form/form.cpp")
    .file("src/sys/common/common.cpp")
    .file("src/sys/search/search.cpp")
    .file("src/sys/wizard/wizard.cpp")
    .file("src/sys/wizard/wizard_gui.cpp")
    .file("src/sys/welcome/welcome.cpp")
    .file("src/sys/welcome/welcome_gui.cpp")
    .file("src/sys/textview/textview.cpp")
    .file("src/sys/textview/textview_gui.cpp")
    .file("src/sys/troubleshooting/troubleshooting.cpp")
    .file("src/sys/troubleshooting/troubleshooting_gui.cpp")
    .file("src/sys/common/mac.mm");
  build.flag("-std=c++17");

  for flag in cpp_flags {
    build.flag(&flag);
  }

  build.compile("espansomodulosys");

  // Render linker flags

  generate_linker_flags(&config_path);

  // On (older) OSX we need to link against the clang runtime,
  // which is hidden in some non-default path.
  //
  // More details at https://github.com/alexcrichton/curl-rust/issues/279.
  if let Some(path) = macos_link_search_path() {
    println!("cargo:rustc-link-lib=clang_rt.osx");
    println!("cargo:rustc-link-search={path}");
  }
}

#[cfg(target_os = "macos")]
fn convert_fat_libraries_to_arm(lib_dir: &Path) {
  for entry in
    glob::glob(&format!("{}/*", lib_dir.to_string_lossy())).expect("failed to glob directory")
  {
    let path = entry.expect("unable to unwrap glob entry");

    // Make sure it's a fat library
    let lipo_output = std::process::Command::new("lipo")
      .args(["-detailed_info", &path.to_string_lossy()])
      .output()
      .expect("unable to check if library is fat");
    let lipo_output = String::from_utf8_lossy(&lipo_output.stdout);
    let lipo_output = lipo_output.trim();
    if !lipo_output.contains("Fat header") {
      println!(
        "skipping {} as it's not a fat library",
        path.to_string_lossy()
      );
      continue;
    }

    println!("converting {} to arm", path.to_string_lossy(),);

    let result = std::process::Command::new("lipo")
      .args([
        "-thin",
        "arm64",
        &path.to_string_lossy(),
        "-output",
        &path.to_string_lossy(),
      ])
      .output()
      .expect("unable to extract arm64 slice from library");

    assert!(
      result.status.success(),
      "unable to convert fat library to arm64 version"
    );
  }
}

#[cfg(not(target_os = "windows"))]
fn get_cpp_flags(wx_config_path: &Path) -> Vec<String> {
  let config_output = std::process::Command::new(wx_config_path)
    .arg("--cxxflags")
    .output()
    .expect("unable to execute wx-config");
  let config_libs =
    String::from_utf8(config_output.stdout).expect("unable to parse wx-config output");
  let cpp_flags: Vec<String> = config_libs
    .split(' ')
    .filter_map(|s| {
      if s.trim().is_empty() {
        None
      } else {
        Some(s.trim().to_owned())
      }
    })
    .collect();
  cpp_flags
}

#[cfg(not(target_os = "windows"))]
fn generate_linker_flags(wx_config_path: &Path) {
  use regex::Regex;
  let config_output = std::process::Command::new(wx_config_path)
    .arg("--libs")
    .output()
    .expect("unable to execute wx-config libs");
  let config_libs =
    String::from_utf8(config_output.stdout).expect("unable to parse wx-config libs output");
  let linker_flags: Vec<String> = config_libs
    .split(' ')
    .filter_map(|s| {
      if s.trim().is_empty() {
        None
      } else {
        Some(s.trim().to_owned())
      }
    })
    .collect();

  let static_lib_extract = Regex::new(r"lib/lib(.*)\.a").unwrap();

  // Translate the flags generated by `wx-config` to commands
  // that cargo can understand.
  for (i, flag) in linker_flags.iter().enumerate() {
    if flag.starts_with("-L") {
      let path = flag.trim_start_matches("-L");
      println!("cargo:rustc-link-search=native={path}");
    } else if flag.starts_with("-framework") {
      println!("cargo:rustc-link-lib=framework={}", linker_flags[i + 1]);
    } else if flag.starts_with('/') {
      let captures = static_lib_extract
        .captures(flag)
        .expect("unable to capture flag regex");
      let libname = captures.get(1).expect("unable to find static libname");
      println!("cargo:rustc-link-lib=static={}", libname.as_str());
    } else if flag.starts_with("-l") {
      let libname = flag.trim_start_matches("-l");
      println!("cargo:rustc-link-lib=dylib={libname}");
    }
  }
}

// Taken from curl-rust: https://github.com/alexcrichton/curl-rust/pull/283/files
#[cfg(target_os = "macos")]
fn macos_link_search_path() -> Option<String> {
  let output = std::process::Command::new("clang")
    .arg("--print-search-dirs")
    .output()
    .ok()?;
  if !output.status.success() {
    println!("failed to run 'clang --print-search-dirs', continuing without a link search path");
    return None;
  }

  let stdout = String::from_utf8_lossy(&output.stdout);
  for line in stdout.lines() {
    if line.contains("libraries: =") {
      let path = line.split('=').nth(1)?;
      return Some(format!("{path}/lib/darwin"));
    }
  }

  println!("failed to determine link search path, continuing without it");
  None
}

// TODO: add documentation for linux
// Install wxWidgets:
// sudo apt install libwxgtk3.0-0v5 libwxgtk3.0-dev
//
// cargo run
#[cfg(target_os = "linux")]
fn build_native() {
  // Make sure wxWidgets is installed
  // Depending on the installation package, the 'wx-config' command might be available under
  // different names, so we need to check them all.
  // See also: https://github.com/espanso/espanso/issues/840
  let possible_wx_config_names = ["wx-config", "wx-config-gtk3", "wx-config-qt"];
  let wx_config_command = possible_wx_config_names
    .iter()
    .find(|&name| {
      std::process::Command::new(name)
        .arg("--version")
        .output()
        .is_ok()
    })
    .unwrap_or_else(|| {
      panic!(
        "wxWidgets is not installed, cannot execute {}",
        possible_wx_config_names.join(" or ")
      )
    });

  let config_path = PathBuf::from(wx_config_command);
  let cpp_flags = get_cpp_flags(&config_path);

  let mut build = cc::Build::new();
  build
    .cpp(true)
    .file("src/sys/form/form.cpp")
    .file("src/sys/search/search.cpp")
    .file("src/sys/common/common.cpp")
    .file("src/sys/wizard/wizard.cpp")
    .file("src/sys/wizard/wizard_gui.cpp")
    .file("src/sys/welcome/welcome.cpp")
    .file("src/sys/welcome/welcome_gui.cpp")
    .file("src/sys/textview/textview.cpp")
    .file("src/sys/textview/textview_gui.cpp")
    .file("src/sys/troubleshooting/troubleshooting.cpp")
    .file("src/sys/troubleshooting/troubleshooting_gui.cpp");
  build.flag("-std=c++17");

  for flag in cpp_flags {
    build.flag(&flag);
  }

  build.compile("espansomodulosys");

  // Render linker flags

  generate_linker_flags(&config_path);
}

fn main() {
  build_native();
}

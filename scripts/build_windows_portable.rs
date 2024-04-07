//! ```cargo
//! [dependencies]
//! glob = "0.3.0"
//! envmnt = "*"
//! fs_extra = "1.3.0"
//! ```

use std::path::PathBuf;

const TARGET_DIR: &str = "target/windows/portable";
const RESOURCE_DIR: &str = "target/windows/resources";

fn main() {
  // Clean the target directory
  let _ = std::fs::remove_dir_all(TARGET_DIR);
  // Create the target directory
  std::fs::create_dir_all(TARGET_DIR).expect("unable to create target directory");
  let target_dir = PathBuf::from(TARGET_DIR);
  if !target_dir.is_dir() {
    panic!("expected target directory, found none");
  }

  let resources_dir = PathBuf::from(RESOURCE_DIR);
  if !resources_dir.is_dir() {
    panic!("expected resources dir, found none");
  }

  // Copy all the resources
  fs_extra::dir::copy(&resources_dir, &target_dir, &fs_extra::dir::CopyOptions {
    content_only: true,
    ..Default::default()
  }).expect("unable to copy resources");

  // Create the launcher
  std::fs::write(target_dir.join("START_ESPANSO.bat"), r#"start espansod.exe launcher"#).unwrap();

  // Create the necessary folders
  std::fs::create_dir_all(target_dir.join(".espanso")).expect("unable to create data directory");
  std::fs::create_dir_all(target_dir.join(".espanso-runtime")).expect("unable to create runtime directory");

  std::fs::write(target_dir.join("README.txt"), r##"Welcome to Espanso (Portable edition)!

To start espanso, you can double click on "START_ESPANSO.bat"  

After the first run, you will see some files in the ".espanso" directory.
This is where your snippets and configurations should be defined.

For more information, please visit the official documentation: 
https://espanso.org/docs/

IMPORTANT: Don't delete any file or directory, otherwise espanso won't work.


FOR ADVANCED USERS:  

Espanso also offers a rich CLI interface. To start it from the terminal, cd into the 
current directory and run "espanso start". You can also run "espanso --help" for more information.

You might have noticed that the directory contains both an "espansod.exe" and an "espanso.cmd" file.
You should generally avoid running "espansod.exe" directly, and instead use the "espanso.cmd"
wrapper (which can simply be run as "espanso" in the terminal). This is needed to correctly manage
STD console handles on Windows.
  "##).unwrap();
}

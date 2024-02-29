//! ```cargo
//! [dependencies]
//! cc = "=1.0.73"
//! glob = "0.3.0"
//! envmnt = "*"
//! ```

use std::path::PathBuf;

const TARGET_DIR: &str = "target/windows/resources";

fn main() {
  // Clean the target directory
  let _ = std::fs::remove_dir_all(TARGET_DIR);
  // Create the target directory
  std::fs::create_dir_all(TARGET_DIR).expect("unable to create target directory");
  let target_dir = PathBuf::from(TARGET_DIR);
  if !target_dir.is_dir() {
    panic!("expected target directory, found none");
  }

  // We first need to find all the DLLs to redistribute with the binary
  // These are found inside the MSVC compiler directory, usually in a place like:
  // C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Redist\MSVC\14.28.29910\x64\Microsoft.VC142.CRT
  // and they include files like vcruntime140.dll and msvcp140.dll

  // First, we try to find the directory containing the various versions:
  // C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Redist\MSVC\
  let tool = cc::windows_registry::find_tool("msvc", "devenv")
    .expect("unable to locate MSVC compiler, did you install Visual Studio?");
  let mut versions_dir = None;
  let mut current_root = tool.path();
  while let Some(parent) = current_root.parent() {
    let target = parent.join("VC").join("Redist").join("MSVC");
    if target.exists() {
      versions_dir = Some(target);
      break;
    }
    current_root = parent;
  }
  let versions_dir = versions_dir.expect(
    r"unable to find path: C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Redist\MSVC",
  );

  // Then we try to find a suitable directory containing the required DLLs
  // C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Redist\MSVC\14.28.29910\x64\Microsoft.VC142.CRT
  let glob_pattern = format!(
    r"{}\*\x64\Microsoft.*\vcruntime140_1.dll",
    versions_dir.to_string_lossy().to_string()
  );
  println!("glob_pattern: {}", glob_pattern);
  let target_file = glob::glob(&glob_pattern)
    .expect("failed to read glob pattern")
    .next()
    .expect("unable to find vcruntime140_1.dll file")
    .expect("unable to extract path of vcruntime140_1.dll file");

  // Copy the DLLs in the target directory
  let parent_dir = target_file
    .parent()
    .expect("unable to obtain directory containing DLLs");
  for entry in glob::glob(&format!(
    r"{}\*.dll",
    parent_dir.to_string_lossy().to_string()
  ))
  .expect("unable to glob over DLLs")
  {
    let entry = entry.expect("unable to unwrap DLL entry");
    let filename = entry.file_name().expect("unable to obtain filename");
    std::fs::copy(&entry, target_dir.join(filename)).expect("unable to copy DLL");
  }

  // Copy the executable
  let exec_path = envmnt::get_or_panic("EXEC_PATH");
  let exec_file = PathBuf::from(&format!("{}.exe", exec_path));
  if !exec_file.is_file() {
    panic!("expected espanso binary, found none in {:?}", exec_file);
  }
  std::fs::copy(exec_file, target_dir.join("espansod.exe")).unwrap();

  // Create the CLI wrapper
  std::fs::write(target_dir.join("espanso.cmd"), r#"@"%~dp0espansod.exe" %*"#).unwrap();
}

//! ```cargo
//! [dependencies]
//! glob = "0.3.0"
//! envmnt = "*"
//! fs_extra = "1.3.0"
//! toml = "0.5.8"
//! dunce = "1.0.2"
//! ```

use std::path::PathBuf;
use std::process::Command;
use toml::Value;

const INSTALLER_NAME: &str = "Espanso-Win-Installer";

const TARGET_DIR: &str = "target/windows/installer";
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

  // Check InnoSetup
  Command::new("iscc").output().expect("Could not find Inno Setup compiler. Please install it from here: http://www.jrsoftware.org/isdl.php");

  // Read the InnoSetup template
  let makefile_path = envmnt::get_or_panic("CARGO_MAKE_MAKEFILE_PATH");
  let makefile_path = PathBuf::from(makefile_path);
  let project_path = makefile_path
    .parent()
    .expect("unable to inferproject directory");
  let script_resources_path = project_path
    .join("scripts")
    .join("resources")
    .join("windows");
  let template_path = script_resources_path.join("setupscript.iss");
  if !template_path.is_file() {
    panic!("InnoSetup template not found");
  }

  let template = std::fs::read_to_string(template_path).expect("unable to read InnoSetup template");

  let espanso_toml_path = project_path.join("espanso").join("Cargo.toml");
  if !espanso_toml_path.is_file() {
    panic!("could not find espanso Cargo.toml file");
  }

  let espanso_toml_str =
    std::fs::read_to_string(espanso_toml_path).expect("unable to read Cargo.toml file");
  let espanso_toml = espanso_toml_str
    .parse::<Value>()
    .expect("unable to parse Cargo.toml");

  let arch = envmnt::get_or_panic("BUILD_ARCH");
  let arch = if arch == "current" {
    std::env::consts::ARCH
  } else {
    &arch
  };

  // Populating template variables
  let mut iss_setup = template;
  iss_setup = iss_setup.replace(
    "{{{app_version}}}",
    espanso_toml["package"].as_table().unwrap()["version"]
      .as_str()
      .unwrap(),
  );
  iss_setup = iss_setup.replace(
    "{{{app_url}}}",
    espanso_toml["package"].as_table().unwrap()["homepage"]
      .as_str()
      .unwrap(),
  );
  iss_setup = iss_setup.replace(
    "{{{app_license}}}",
    &dunce::canonicalize(project_path.join("LICENSE"))
      .unwrap()
      .to_string_lossy()
      .to_string(),
  );
  iss_setup = iss_setup.replace(
    "{{{app_icon}}}",
    &dunce::canonicalize(script_resources_path.join("icon.ico"))
      .unwrap()
      .to_string_lossy()
      .to_string(),
  );
  iss_setup = iss_setup.replace(
    "{{{cli_helper}}}",
    &dunce::canonicalize(script_resources_path.join("espanso.cmd"))
      .unwrap()
      .to_string_lossy()
      .to_string(),
  );
  iss_setup = iss_setup.replace(
    "{{{output_dir}}}",
    &dunce::canonicalize(TARGET_DIR)
      .unwrap()
      .to_string_lossy()
      .to_string(),
  );
  iss_setup = iss_setup.replace("{{{output_name}}}", &format!("{}-{}", INSTALLER_NAME, arch));
  iss_setup = iss_setup.replace(
    "{{{executable_path}}}",
    &dunce::canonicalize(&resources_dir.join("espansod.exe"))
      .unwrap()
      .to_string_lossy()
      .to_string(),
  );

  // Generate file includes
  let mut include_paths = Vec::new();
  for entry in glob::glob(&format!(
    r"{}\*.dll",
    resources_dir.to_string_lossy().to_string()
  ))
  .expect("unable to glob over DLLs")
  {
    let entry = entry.expect("unable to unwrap DLL entry");
    include_paths.push(format!(
      "Source: \"{}\"; DestDir: \"{{app}}\"; Flags: ignoreversion",
      dunce::canonicalize(&entry)
        .unwrap()
        .to_string_lossy()
        .to_string()
    ));
  }

  iss_setup = iss_setup.replace("{{{dll_include}}}", &include_paths.join("\r\n"));

  let iss_setup_path = target_dir.join("setupscript.iss");
  std::fs::write(&iss_setup_path, &iss_setup).expect("unable to write InnoSetup setup script");

  // Compile the installer

  let status = Command::new("iscc")
    .arg(&iss_setup_path.to_string_lossy().to_string())
    .status()
    .expect("unable to invoke InnoSetup compilation");
  if !status.success() {
    panic!("InnoSetup compilation process returned non-zero exit code");
  }
}

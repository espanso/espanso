//! ```cargo
//! [dependencies]
//! glob = "0.3.0"
//! fs_extra = "1.3.0"
//! base64 = "0.13.0"
//! anyhow = "1.0.38"
//! ```

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

const WINDOWS_KITS_LOCATION: &str = "C:/Program Files (x86)/Windows Kits/10/bin";
const CERTIFICATE_TARGET_DIR: &str = "target/codesign";

fn main() {
  let _ = std::fs::remove_dir_all(CERTIFICATE_TARGET_DIR);
  std::fs::create_dir_all(CERTIFICATE_TARGET_DIR).expect("unable to create target directory");
  let certificate_target_dir = PathBuf::from(CERTIFICATE_TARGET_DIR);
  if !certificate_target_dir.is_dir() {
    panic!("expected target directory, found none");
  }

  let signtool_path = get_signtool_location().expect("unable to locate signtool exe");
  println!("using signtool location: {:?}", signtool_path);

  let target_exe_file =
    std::env::var("TARGET_SIGNTOOL_FILE").expect("TARGET_SIGNTOOL_FILE env variable not found");
  let target_exe_path = PathBuf::from(target_exe_file);
  if !target_exe_path.is_file() {
    panic!(
      "target file '{}' cannot be found",
      target_exe_path.display()
    );
  }

  println!("signing file: {:?}", target_exe_path);

  let certificate_pwd = std::env::var("CODESIGN_PWD").expect("CODESIGN_PWD env variable not found");
  let cross_signed_certificate_b64 = std::env::var("CODESIGN_CROSS_SIGNED_B64")
    .expect("CODESIGN_CROSS_SIGNED_B64 env variable not found");
  let codesign_certificate_b64 = std::env::var("CODESIGN_CERTIFICATE_B64")
    .expect("CODESIGN_CERTIFICATE_B64 env variable not found");

  let cross_signed_certificate = DecodedCertificate::new(
    &cross_signed_certificate_b64,
    certificate_target_dir.join("SectigoPublicCodeSigningRootR46_AAA.crt"),
  )
  .expect("unable to decode intermediate cross-signed certificate");
  let codesign_certificate = DecodedCertificate::new(
    &codesign_certificate_b64,
    certificate_target_dir.join("codesign.pfx"),
  )
  .expect("unable to decode codesign certificate");

  let mut cmd = Command::new(signtool_path);
  cmd.args(&[
    "sign",
    "/fd",
    "SHA256",
    "/p",
    &certificate_pwd,
    "/ac",
    &cross_signed_certificate.path(),
    "/f",
    &codesign_certificate.path(),
    "/tr",
    "http://timestamp.sectigo.com/rfc3161",
    "/td",
    "sha256",
    &target_exe_path.to_string_lossy(),
  ]);

  let mut handle = cmd.spawn().expect("signtool spawn failed");
  let result = handle.wait().expect("unable to read signtool exit status");
  if !result.success() {
    panic!("signtool failed");
  }
}

// Inspired by: https://github.com/dlemstra/code-sign-action/blob/main/index.ts#L143
fn get_signtool_location() -> Option<PathBuf> {
  let mut path: Option<PathBuf> = None;
  let mut max_version = 0;
  for entry in glob::glob(&format!("{}/*", WINDOWS_KITS_LOCATION))
    .expect("unable to glob windows kits location")
  {
    let entry = entry.expect("unable to unwrap glob entry");
    if !entry.is_dir() {
      continue;
    }
    if !entry.to_string_lossy().ends_with(".0") {
      continue;
    }
    let folder_name = entry.file_name().expect("unable to extract folder_name");
    let folder_version_str = folder_name.to_string_lossy().replace(".", "");
    let folder_version = folder_version_str
      .parse::<i32>()
      .expect("invalid folder version string");
    if folder_version > max_version {
      let signtool_path_candidate = entry.join("x86").join("signtool.exe");
      if signtool_path_candidate.is_file() {
        path = Some(signtool_path_candidate);
        max_version = folder_version;
      }
    }
  }

  return path;
}

struct DecodedCertificate {
  decoded_file: PathBuf,
}

impl DecodedCertificate {
  pub fn new(b64: &str, target_file: PathBuf) -> Result<Self> {
    // Keys and certificates are encoded with whitespaces/newlines in them, but we need to remove them
    let filtered_b64: String = b64.chars().filter(|c| !c.is_whitespace()).collect();
    let decoded = base64::decode(filtered_b64)?;
    std::fs::write(&target_file, &decoded)?;
    Ok(Self {
      decoded_file: target_file,
    })
  }

  pub fn path(&self) -> String {
    self.decoded_file.to_string_lossy().to_string()
  }
}

// Delete the certificate files when they are not needed anymore
// to minimize the attack surface
impl Drop for DecodedCertificate {
  fn drop(&mut self) {
    std::fs::remove_file(&self.decoded_file).expect("unable to remove certificate file")
  }
}

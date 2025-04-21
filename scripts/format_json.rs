use std::process::Command;

fn main() {
    for path in [
  ".devcontainer",
  ".github",
  ".vscode",
  "espanso",
  "espanso-detect",
  "espanso-ui",
  "espanso-inject",
  "espanso-ipc",
  "espanso-config",
  "espanso-match",
  "espanso-clipboard",
  "espanso-render",
  "espanso-info",
  "espanso-modulo",
  "espanso-mac-utils",
  "espanso-kvs",
  "espanso-engine",
  "espanso-package",
  "biome.json",
  ] {
        println!("{}", String::from_utf8(Command::new("biome")
            .args(["format", "--write", path])
            .output()
            .expect("Failed to execute biome")
            .stderr).unwrap()
        );
    }
}

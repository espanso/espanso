//! ```cargo
//! [dependencies]
//! glob = "0.3.0"
//! ```

use glob::glob;
use std::process::Command;

fn main() {
    for dir in [
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
    ] {
        for ext in ["c", "h", "cc", "hh", "cpp"] {
            // ignore
            for entry in glob(format!("{}/**/*.{}", dir, ext).as_str())
                .expect("Failed to read the glob pattern")
            {
                match entry {
                    Ok(path) => println!("{}", String::from_utf8(Command::new("clang-format")
                            .args(["-i", "--verbose", path.to_str().expect("Failed to convert path to a string")])
                            .output()
                            .expect("Failed to execute clang-format")
                            .stderr).unwrap()
                    ),
                    Err(e) => println!("{:?}", e),
                }
            }
        }
    }
}

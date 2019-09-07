use std::process::{Command, Stdio};
use std::io::{Write};

pub struct LinuxClipboardManager {

}

impl super::ClipboardManager for LinuxClipboardManager {
    fn initialize(&self) {
        // TODO: check if xclip is present and log an error otherwise.
    }

    fn get_clipboard(&self) -> Option<String>  {
        let res = Command::new("xclip")
            .args(&["-o", "-sel", "clip"])
            .output();

        if let Ok(output) = res {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                return Some((*s).to_owned());
            }
        }

        None
    }

    fn set_clipboard(&self, payload: &str) {
        let res = Command::new("xclip")
            .args(&["-sel", "clip"])
            .stdin(Stdio::piped())
            .spawn();

        if let Ok(mut child) = res {
            let stdin = child.stdin.as_mut();

            if let Some(output) = stdin {
                let res = output.write_all(payload.as_bytes());

                if let Err(_) = res {
                    // TODO: log error
                }
            }
        }
    }
}

impl LinuxClipboardManager {

}
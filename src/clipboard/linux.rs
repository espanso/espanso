use std::process::{Command, ExitStatus, Stdio};
use std::io::{Write, Read};

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
        let mut res = Command::new("xclip")
            .args(&["-sel", "clip"])
            .stdin(Stdio::piped())
            .spawn();

        if let Ok(mut child) = res {
            let mut stdin = child.stdin.as_mut();

            if let Some(mut output) = stdin {
                output.write_all(payload.as_bytes());

//                let status = child.wait();
//
//                if let Ok(status) = status {
//                    if !status.success() {
//                        //TODO: log error
//                    }
//                }
            }
        }
    }
}

impl LinuxClipboardManager {

}
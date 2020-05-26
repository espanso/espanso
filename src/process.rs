/*
 * This file is part of espanso.
 *
 * Copyright (C) 2020 Federico Terzi
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

use log::warn;
use std::process::{Command, Stdio, Child};
use widestring::WideCString;
use std::io;

#[cfg(target_os = "windows")]
pub fn spawn_process(cmd: &str, args: &Vec<String>) {
    // TODO: modify with https://doc.rust-lang.org/std/os/windows/process/trait.CommandExt.html
    let quoted_args: Vec<String> = args.iter().map(|arg| format!("\"{}\"", arg)).collect();
    let quoted_args = quoted_args.join(" ");
    let final_cmd = format!("\"{}\" {}", cmd, quoted_args);
    unsafe {
        let cmd_wstr = WideCString::from_str(&final_cmd);
        if let Ok(string) = cmd_wstr {
            let res = crate::bridge::windows::start_process(string.as_ptr());
            if res < 0 {
                warn!("unable to start process: {}", final_cmd);
            }
        } else {
            warn!("unable to convert process string into wide format")
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn spawn_process(cmd: &str, args: &Vec<String>) -> io::Result<Child> {
    Command::new(cmd).args(args).spawn()
}

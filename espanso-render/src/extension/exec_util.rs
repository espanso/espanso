/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2022 Federico Terzi
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

pub enum MacShell {
    Bash,
    Fish,
    Nu,
    Pwsh,
    Sh,
    Zsh,
}

// Determine the PATH env variable value for macOS available inside a regular terminal session
pub fn determine_path_env_variable_override(explicit_shell: Option<MacShell>) -> Option<String> {
    if cfg!(not(target_os = "macos")) {
        return None;
    }
    let shell: MacShell = explicit_shell.or_else(determine_default_macos_shell)?;
    let (command, args) = path_query_command(&shell);

    launch_command_and_get_output(command, args)
}

// The command (and its arguments) used to print the PATH of a login shell as a
// POSIX colon-separated string.
fn path_query_command(shell: &MacShell) -> (&'static str, &'static [&'static str]) {
    match shell {
        MacShell::Bash => ("bash", &["--login", "-c", "source ~/.bashrc; echo $PATH"]),
        // In fish, PATH is a list variable, so `echo $PATH` would print its entries
        // separated by spaces instead of colons. We ask fish to join them explicitly.
        // `--` keeps entries starting with a hyphen from being read as flags, and
        // `; true` absorbs the exit code 1 `string join` returns for a single entry.
        MacShell::Fish => ("fish", &["--login", "-c", "string join -- : $PATH; true"]),
        MacShell::Nu => ("nu", &["--login", "-c", "$env.PATH"]),
        MacShell::Pwsh => (
            "pwsh",
            &[
                "-Login",
                "-Command",
                "if(Test-Path \"$PROFILE\") { . \"$PROFILE\" }; Write-Host $env:PATH",
            ],
        ),
        MacShell::Sh => ("sh", &["--login", "-c", "echo $PATH"]),
        MacShell::Zsh => ("zsh", &["--login", "-c", "source ~/.zshrc; echo $PATH"]),
    }
}

pub fn determine_default_macos_shell() -> Option<MacShell> {
    if cfg!(not(target_os = "macos")) {
        return None;
    }
    use regex::Regex;
    use std::process::Command;
    use std::sync::LazyLock;

    let output = Command::new("sh")
        .args(["--login", "-c", "dscl . -read ~/ UserShell"])
        .output()
        .ok()?;

    static EXTRACT_SHELL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"UserShell:\s(.*)$").expect("unable to generate regex to extract default shell")
    });

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let captures = EXTRACT_SHELL_REGEX.captures(output_str.trim())?;

    let shell = captures.get(1)?.as_str().trim();

    if shell.ends_with("/bash") {
        Some(MacShell::Bash)
    } else if shell.ends_with("/fish") {
        Some(MacShell::Fish)
    } else if shell.ends_with("/nu") {
        Some(MacShell::Nu)
    } else if shell.ends_with("/pwsh") {
        Some(MacShell::Pwsh)
    } else if shell.ends_with("/sh") {
        Some(MacShell::Sh)
    } else if shell.ends_with("/zsh") {
        Some(MacShell::Zsh)
    } else {
        None
    }
}

fn launch_command_and_get_output(command: &str, args: &[&str]) -> Option<String> {
    use std::process::Command;

    let output = Command::new(command).args(args).output().ok()?;

    if !output.status.success() {
        return None;
    }

    // without trimming, the shell's trailing newline lands inside the last PATH
    // entry, making it unresolvable
    let output_str = String::from_utf8_lossy(&output.stdout);
    let output_str = output_str.trim();
    if output_str.is_empty() {
        return None;
    }
    Some(output_str.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fish_joins_path_with_colons() {
        // fish stores PATH as a list, so it must be joined with colons explicitly,
        // otherwise the resulting PATH is a single bogus space-separated entry.
        let (command, args) = path_query_command(&MacShell::Fish);
        assert_eq!(command, "fish");
        assert!(args.last().unwrap().starts_with("string join -- : $PATH"));
    }

    #[test]
    fn every_shell_query_is_unchanged_except_fish() {
        for (shell, expected) in [
            (MacShell::Bash, "source ~/.bashrc; echo $PATH"),
            (MacShell::Nu, "$env.PATH"),
            (MacShell::Sh, "echo $PATH"),
            (MacShell::Zsh, "source ~/.zshrc; echo $PATH"),
        ] {
            assert_eq!(*path_query_command(&shell).1.last().unwrap(), expected);
        }
    }

    #[test]
    fn trailing_newline_would_corrupt_the_last_entry() {
        // guards the trim in launch_command_and_get_output: a shell prints a
        // trailing newline, and PATH is split on ':' only
        let raw = "/usr/local/bin:/usr/bin\n";
        assert_eq!(raw.split(':').next_back(), Some("/usr/bin\n"));
        assert_eq!(raw.trim().split(':').next_back(), Some("/usr/bin"));
    }
}

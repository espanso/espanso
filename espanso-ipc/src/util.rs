/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019-2021 Federico Terzi
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

use anyhow::Result;

// Named pipes are machine-wide, so a pipe named after the ID alone is taken by the first user
// on a multi-user host (Remote Desktop, AVD) and every other user's espanso fails with "Access is denied".
// Deriving the name from the per-user runtime dir scopes it the same way the Unix socket is scoped.
// FNV-1a rather than DefaultHasher, whose algorithm is unspecified and may change between Rust releases,
// so that builds from different toolchains agree.
#[cfg(any(target_os = "windows", test))]
pub fn pipe_name(id: &str, parent_dir: &std::path::Path) -> String {
    let key = parent_dir.to_string_lossy().to_lowercase();
    let hash = key.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!("\\\\.\\pipe\\{id}-{hash:016x}")
}

// Unbuffered version, necessary to concurrently write
// to the buffer if necessary (when receiving sync messages)
pub fn read_line<R: std::io::Read>(stream: R) -> Result<Option<String>> {
    let mut buffer = Vec::new();

    let mut is_eof = true;

    #[allow(clippy::unbuffered_bytes)]
    for byte_res in stream.bytes() {
        let byte = byte_res?;

        if byte == 10 {
            // Newline
            break;
        }
        buffer.push(byte);
        is_eof = false;
    }

    if is_eof {
        Ok(None)
    } else {
        Ok(Some(String::from_utf8(buffer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::pipe_name;
    use std::path::Path;

    #[test]
    fn pipe_name_is_scoped_to_the_runtime_dir() {
        let alice = pipe_name(
            "espansodaemonv2",
            Path::new(r"C:\Users\Alice\AppData\Local\espanso"),
        );
        let bob = pipe_name(
            "espansodaemonv2",
            Path::new(r"C:\Users\Bob\AppData\Local\espanso"),
        );
        let alice_again = pipe_name(
            "espansodaemonv2",
            Path::new(r"c:\users\alice\AppData\Local\espanso"),
        );

        assert_ne!(alice, bob);
        assert_eq!(alice, alice_again);
        assert_ne!(
            alice,
            pipe_name(
                "espansoworkerv2",
                Path::new(r"C:\Users\Alice\AppData\Local\espanso")
            )
        );
        assert!(alice.starts_with(r"\\.\pipe\espansodaemonv2-"));
        assert!(!alice[r"\\.\pipe\".len()..].contains('\\'));
    }
}

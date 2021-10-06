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

// Unbuffered version, necessary to concurrently write
// to the buffer if necessary (when receiving sync messages)
pub fn read_line<R: std::io::Read>(stream: R) -> Result<Option<String>> {
  let mut buffer = Vec::new();

  let mut is_eof = true;

  for byte_res in stream.bytes() {
    let byte = byte_res?;

    if byte == 10 {
      // Newline
      break;
    } else {
      buffer.push(byte);
    }

    is_eof = false;
  }

  if is_eof {
    Ok(None)
  } else {
    Ok(Some(String::from_utf8(buffer)?))
  }
}

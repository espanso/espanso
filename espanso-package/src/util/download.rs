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

use anyhow::{Context, Result};
use std::io::{copy, Cursor};
use std::path::Path;

pub fn download_and_extract_zip(url: &str, dest_dir: &Path) -> Result<()> {
  let data = download(url).context("error downloading archive")?;
  extract_zip(data, dest_dir).context("error extracting archive")
}

fn download(url: &str) -> Result<Vec<u8>> {
  let client = reqwest::blocking::Client::builder();
  let client = client.build()?;

  let mut response = client.get(url).send()?;

  let mut buffer = Vec::new();
  copy(&mut response, &mut buffer)?;
  Ok(buffer)
}

// Adapted from zip-rs extract.rs example
fn extract_zip(data: Vec<u8>, dest_dir: &Path) -> Result<()> {
  let reader = Cursor::new(data);

  let mut archive = zip::ZipArchive::new(reader)?;

  for i in 0..archive.len() {
    let mut file = archive.by_index(i)?;
    let outpath = match file.enclosed_name() {
      Some(path) => dest_dir.join(path),
      None => continue,
    };

    if (&*file.name()).ends_with('/') {
      std::fs::create_dir_all(&outpath)?;
    } else {
      if let Some(p) = outpath.parent() {
        if !p.exists() {
          std::fs::create_dir_all(&p)?;
        }
      }
      let mut outfile = std::fs::File::create(&outpath)?;
      copy(&mut file, &mut outfile)?;
    }
  }

  Ok(())
}

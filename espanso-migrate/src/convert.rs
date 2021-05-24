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

use std::{cmp::Ordering, collections::HashMap, path::PathBuf};
use yaml_rust::yaml::Hash;

pub fn convert(input_files: HashMap<String, Hash>) -> HashMap<String, Hash> {
  let mut output_files = HashMap::new();

  let sorted_input_files = sort_input_files(&input_files);

  for input_path in sorted_input_files {
    let yaml = input_files
      .get(&input_path)
      .expect("received unexpected file in input function");

    if let Some((file_name, file_name_without_extension)) = extract_name_information(&input_path) {
      println!("file: {}, {}", file_name, file_name_without_extension);

      // TODO: execute the actual conversion

    } else {
      eprintln!("unable to extract filename from path: {}", input_path);
    }
  }

  output_files
}

fn sort_input_files(input_files: &HashMap<String, Hash>) -> Vec<String> {
  let mut files: Vec<String> = input_files.iter().map(|(key, _)| key.clone()).collect();
  files.sort_by(|f1, f2| {
    let f1_slashes = f1.matches("/").count();
    let f2_slashes = f2.matches("/").count();
    if f1_slashes > f2_slashes {
      Ordering::Greater
    } else if f1_slashes < f2_slashes {
      Ordering::Less
    } else {
      f1.cmp(f2)
    }
  });
  files
}

fn extract_name_information(path: &str) -> Option<(String, String)> {
  let path_buf = PathBuf::from(path);
  let file_name = path_buf.file_name()?.to_string_lossy().to_string();
  let extension = path_buf.extension()?.to_string_lossy().to_string();
  let file_name_without_extension = file_name.trim_end_matches(&format!(".{}", extension)).to_string();
  Some((file_name, file_name_without_extension))
}
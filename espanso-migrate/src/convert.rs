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
use yaml_rust::{yaml::Hash, Yaml, YamlEmitter};

pub fn convert(input_files: HashMap<String, Hash>) -> HashMap<String, Hash> {
  let mut output_files = HashMap::new();

  let sorted_input_files = sort_input_files(&input_files);

  let mut config_names_to_path = HashMap::new();

  for input_path in sorted_input_files {
    let yaml = input_files
      .get(&input_path)
      .expect("received unexpected file in input function");

    let yaml_matches = yaml_get_vec(yaml, "matches");
    let yaml_global_vars = yaml_get_vec(yaml, "global_vars");

    let yaml_parent = yaml_get_string(yaml, "parent");
    let yaml_name = yaml_get_string(yaml, "name");

    let should_generate_match = yaml_matches.is_some() || yaml_global_vars.is_some();
    if should_generate_match {
      let should_underscore = !input_path.starts_with("default") && yaml_parent != Some("default");
      let match_output_path = calculate_output_match_path(&input_path, should_underscore);
      if match_output_path.is_none() {
        eprintln!(
          "unable to determine output path for {}, skipping...",
          input_path
        );
        continue;
      }
      let match_output_path = match_output_path.unwrap();

      if let Some(name) = yaml_name {
        config_names_to_path.insert(name.to_string(), match_output_path.clone());
      }

      let output_yaml = output_files.entry(match_output_path).or_insert(Hash::new());

      if let Some(global_vars) = yaml_global_vars {
        let output_global_vars = output_yaml
          .entry(Yaml::String("global_vars".to_string()))
          .or_insert(Yaml::Array(Vec::new()));
        if let Yaml::Array(out_global_vars) = output_global_vars {
          out_global_vars.extend(global_vars.clone());
        } else {
          eprintln!("unable to transform global_vars for file: {}", input_path);
        }
      }

      if let Some(matches) = yaml_matches {
        let output_matches = output_yaml
          .entry(Yaml::String("matches".to_string()))
          .or_insert(Yaml::Array(Vec::new()));
        if let Yaml::Array(out_matches) = output_matches {
          out_matches.extend(matches.clone());
        } else {
          eprintln!("unable to transform matches for file: {}", input_path);
        }
      }
    }

    let yaml_filter_class = yaml_get_string(yaml, "filter_class");
    let yaml_filter_title = yaml_get_string(yaml, "filter_title");
    let yaml_filter_exec = yaml_get_string(yaml, "filter_exec");

    let should_generate_config = input_path.starts_with("default")
      || yaml_filter_class.is_some()
      || yaml_filter_exec.is_some()
      || yaml_filter_title.is_some();

    if should_generate_config {
      let config_output_path = calculate_output_config_path(&input_path);

      let mut output_yaml = Hash::new();

      copy_field_if_present(yaml, "filter_title", &mut output_yaml, "filter_title");
      copy_field_if_present(yaml, "filter_class", &mut output_yaml, "filter_class");
      copy_field_if_present(yaml, "filter_exec", &mut output_yaml, "filter_exec");

      // TODO: copy other config fields: https://github.com/federico-terzi/espanso/blob/master/src/config/mod.rs#L169

      // TODO: if a match file was created above of type "underscored", then explicitly include it here
      // depending on whether "exclude_default_entries" is set, use "includes" or "extra_includes"

      output_files.insert(config_output_path, output_yaml);
    }

    // TODO: create config file

    // TODO: execute the actual conversion
  }

  // TODO: here resolve parent: name imports

  // TODO: remove this prints
  for (file, content) in output_files {
    let mut out_str = String::new();
    {
      let mut emitter = YamlEmitter::new(&mut out_str);
      emitter.dump(&Yaml::Hash(content)).unwrap(); // dump the YAML object to a String
    }
    println!("\n------- {} ------------\n{}", file, out_str);
  }

  todo!();
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

// TODO: test
fn calculate_output_match_path(path: &str, is_underscored: bool) -> Option<String> {
  let path_buf = PathBuf::from(path);
  let file_name = path_buf.file_name()?.to_string_lossy().to_string();

  let path = if is_underscored {
    path.replace(&file_name, &format!("_{}", file_name))
  } else {
    path.to_string()
  };

  Some(if path.starts_with("user/") {
    format!("match/{}", path.trim_start_matches("user/"))
  } else if path.starts_with("packages/") {
    format!("match/packages/{}", path.trim_start_matches("packages/"))
  } else if path == "default.yml" {
    "match/base.yml".to_string()
  } else {
    format!("match/{}", path)
  })
}

// TODO: test
fn calculate_output_config_path(path: &str) -> String {
  if path.starts_with("user/") {
    format!("config/{}", path.trim_start_matches("user/"))
  } else if path.starts_with("packages/") {
    format!("config/packages/{}", path.trim_start_matches("packages/"))
  } else {
    format!("config/{}", path)
  }
}

fn yaml_get_vec<'a>(yaml: &'a Hash, name: &str) -> Option<&'a Vec<Yaml>> {
  yaml
    .get(&Yaml::String(name.to_string()))
    .and_then(|v| v.as_vec())
}

fn yaml_get_string<'a>(yaml: &'a Hash, name: &str) -> Option<&'a str> {
  yaml
    .get(&Yaml::String(name.to_string()))
    .and_then(|v| v.as_str())
}

fn copy_field_if_present(
  input_yaml: &Hash,
  input_field_name: &str,
  output_yaml: &mut Hash,
  output_field_name: &str,
) {
  if let Some(value) = input_yaml.get(&Yaml::String(input_field_name.to_string())) {
    output_yaml.insert(Yaml::String(output_field_name.to_string()), value.clone());
  }
}

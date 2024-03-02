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

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::{cmp::Ordering, collections::HashMap, path::PathBuf};
use yaml_rust::{yaml::Hash, Yaml};

pub struct ConvertedFile {
  pub origin: String,
  pub content: Hash,
}

pub fn convert(input_files: HashMap<String, Hash>) -> HashMap<String, ConvertedFile> {
  let mut output_files = HashMap::new();

  let sorted_input_files = sort_input_files(&input_files);

  for input_path in sorted_input_files {
    let yaml = input_files
      .get(&input_path)
      .expect("received unexpected file in input function");

    let yaml_matches = yaml_get_vec(yaml, "matches");
    let yaml_global_vars = yaml_get_vec(yaml, "global_vars");

    let yaml_parent = yaml_get_string(yaml, "parent");

    if let Some(parent) = yaml_parent {
      if parent != "default" {
        eprintln!(
          "WARNING: nested 'parent' instructions are not currently supported by the migration tool"
        );
      }
    }

    let should_generate_match = yaml_matches.is_some() || yaml_global_vars.is_some();
    let match_file_path_if_unlisted = if should_generate_match {
      let should_underscore = !input_path.starts_with("default") && yaml_parent != Some("default");
      let match_output_path = calculate_output_match_path(&input_path, should_underscore);
      if match_output_path.is_none() {
        eprintln!("unable to determine output path for {input_path}, skipping...");
        continue;
      }
      let match_output_path = match_output_path.unwrap();

      let output_yaml = output_files
        .entry(match_output_path.clone())
        .or_insert(ConvertedFile {
          origin: input_path.to_string(),
          content: Hash::new(),
        });

      if let Some(global_vars) = yaml_global_vars {
        let mut patched_global_vars: Vec<Yaml> = global_vars.clone();
        patched_global_vars
          .iter_mut()
          .for_each(apply_form_syntax_patch_to_variable);

        let output_global_vars = output_yaml
          .content
          .entry(Yaml::String("global_vars".to_string()))
          .or_insert(Yaml::Array(Vec::new()));
        if let Yaml::Array(out_global_vars) = output_global_vars {
          out_global_vars.extend(patched_global_vars);
        } else {
          eprintln!("unable to transform global_vars for file: {input_path}");
        }
      }

      if let Some(matches) = yaml_matches {
        let mut patched_matches = matches.clone();
        apply_form_syntax_patch(&mut patched_matches);

        let output_matches = output_yaml
          .content
          .entry(Yaml::String("matches".to_string()))
          .or_insert(Yaml::Array(Vec::new()));
        if let Yaml::Array(out_matches) = output_matches {
          out_matches.extend(patched_matches);
        } else {
          eprintln!("unable to transform matches for file: {input_path}");
        }
      }

      if should_underscore {
        Some(match_output_path)
      } else {
        None
      }
    } else {
      None
    };

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
      copy_field_if_present(yaml, "enable_active", &mut output_yaml, "enable");
      copy_field_if_present(yaml, "backend", &mut output_yaml, "backend");
      map_field_if_present(
        yaml,
        "paste_shortcut",
        &mut output_yaml,
        "paste_shortcut",
        |val| match val {
          Yaml::String(shortcut) if shortcut == "CtrlV" => Some(Yaml::String("CTRL+V".to_string())),
          Yaml::String(shortcut) if shortcut == "CtrlShiftV" => {
            Some(Yaml::String("CTRL+SHIFT+V".to_string()))
          }
          Yaml::String(shortcut) if shortcut == "ShiftInsert" => {
            Some(Yaml::String("SHIFT+INSERT".to_string()))
          }
          Yaml::String(shortcut) if shortcut == "CtrlAltV" => {
            Some(Yaml::String("CTRL+ALT+V".to_string()))
          }
          Yaml::String(shortcut) if shortcut == "MetaV" => Some(Yaml::String("META+V".to_string())),
          Yaml::String(_) => None,
          _ => None,
        },
      );
      //copy_field_if_present(yaml, "secure_input_watcher_enabled", &mut output_yaml, "secure_input_watcher_enabled");
      //copy_field_if_present(yaml, "secure_input_watcher_interval", &mut output_yaml, "secure_input_watcher_interval");
      //copy_field_if_present(yaml, "config_caching_interval", &mut output_yaml, "config_caching_interval");
      //copy_field_if_present(yaml, "use_system_agent", &mut output_yaml, "use_system_agent");

      copy_field_if_present(
        yaml,
        "secure_input_notification",
        &mut output_yaml,
        "secure_input_notification",
      );
      copy_field_if_present(yaml, "toggle_interval", &mut output_yaml, "toggle_interval");
      copy_field_if_present(yaml, "toggle_key", &mut output_yaml, "toggle_key");
      copy_field_if_present(
        yaml,
        "preserve_clipboard",
        &mut output_yaml,
        "preserve_clipboard",
      );
      copy_field_if_present(yaml, "backspace_limit", &mut output_yaml, "backspace_limit");
      map_field_if_present(
        yaml,
        "fast_inject",
        &mut output_yaml,
        "disable_x11_fast_inject",
        |val| match val {
          Yaml::Boolean(false) => Some(Yaml::Boolean(true)),
          Yaml::Boolean(true) => Some(Yaml::Boolean(false)),
          _ => None,
        },
      );

      copy_field_if_present(yaml, "auto_restart", &mut output_yaml, "auto_restart");
      copy_field_if_present(yaml, "undo_backspace", &mut output_yaml, "undo_backspace");
      copy_field_if_present(yaml, "show_icon", &mut output_yaml, "show_icon");
      copy_field_if_present(
        yaml,
        "show_notifications",
        &mut output_yaml,
        "show_notifications",
      );
      copy_field_if_present(yaml, "inject_delay", &mut output_yaml, "inject_delay");
      copy_field_if_present(
        yaml,
        "restore_clipboard_delay",
        &mut output_yaml,
        "restore_clipboard_delay",
      );
      copy_field_if_present(yaml, "backspace_delay", &mut output_yaml, "key_delay");
      copy_field_if_present(yaml, "word_separators", &mut output_yaml, "word_separators");

      if yaml
        .get(&Yaml::String("enable_passive".to_string()))
        .is_some()
      {
        eprintln!("WARNING: passive-mode directives were detected, but passive-mode is not supported anymore.");
        eprintln!("Please follow this issue to discover the alternatives: https://github.com/espanso/espanso/issues/540");
      }

      // Link any unlisted match file (the ones starting with the _ underscore, which are excluded by the
      // default.yml config) explicitly, if present.
      if let Some(match_file_path) = match_file_path_if_unlisted {
        let yaml_exclude_default_entries =
          yaml_get_bool(yaml, "exclude_default_entries").unwrap_or(false);
        let key_name = if yaml_exclude_default_entries {
          "includes"
        } else {
          "extra_includes"
        };

        let includes = vec![Yaml::String(format!("../{match_file_path}"))];

        output_yaml.insert(Yaml::String(key_name.to_string()), Yaml::Array(includes));
      }

      output_files.insert(
        config_output_path,
        ConvertedFile {
          origin: input_path,
          content: output_yaml,
        },
      );
    }
  }

  output_files
}

fn sort_input_files(input_files: &HashMap<String, Hash>) -> Vec<String> {
  let mut files: Vec<String> = input_files.iter().map(|(key, _)| key.clone()).collect();
  files.sort_by(|f1, f2| {
    let f1_slashes = f1.matches('/').count();
    let f2_slashes = f2.matches('/').count();
    #[allow(clippy::comparison_chain)]
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
    path.replace(&file_name, &format!("_{file_name}"))
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
    format!("match/{path}")
  })
}

// TODO: test
fn calculate_output_config_path(path: &str) -> String {
  if path.starts_with("user/") {
    format!("config/{}", path.trim_start_matches("user/"))
  } else if path.starts_with("packages/") {
    format!("config/packages/{}", path.trim_start_matches("packages/"))
  } else {
    format!("config/{path}")
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

fn yaml_get_bool(yaml: &Hash, name: &str) -> Option<bool> {
  yaml
    .get(&Yaml::String(name.to_string()))
    .and_then(Yaml::as_bool)
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

fn map_field_if_present(
  input_yaml: &Hash,
  input_field_name: &str,
  output_yaml: &mut Hash,
  output_field_name: &str,
  transform: impl FnOnce(&Yaml) -> Option<Yaml>,
) {
  if let Some(value) = input_yaml.get(&Yaml::String(input_field_name.to_string())) {
    let transformed = transform(value);
    if let Some(transformed) = transformed {
      output_yaml.insert(Yaml::String(output_field_name.to_string()), transformed);
    } else {
      eprintln!("could not convert value for field: {input_field_name}");
    }
  }
}

// This is needed to convert the old form's {{control}} syntax to the new [[control]] one.
fn apply_form_syntax_patch(matches: &mut [Yaml]) {
  for m in matches.iter_mut() {
    if let Yaml::Hash(fields) = m {
      if let Some(Yaml::String(form_option)) = fields.get_mut(&Yaml::String("form".to_string())) {
        let converted = replace_legacy_form_syntax_with_new_one(form_option);
        if &converted != form_option {
          form_option.clear();
          form_option.push_str(&converted);
        }
      }

      if let Some(Yaml::Array(vars)) = fields.get_mut(&Yaml::String("vars".to_string())) {
        vars
          .iter_mut()
          .for_each(apply_form_syntax_patch_to_variable);
      }
    }
  }
}

fn apply_form_syntax_patch_to_variable(variable: &mut Yaml) {
  if let Yaml::Hash(fields) = variable {
    if let Some(Yaml::String(var_type)) = fields.get(&Yaml::String("type".to_string())) {
      if var_type != "form" {
        return;
      }
    }

    if let Some(Yaml::Hash(params)) = fields.get_mut(&Yaml::String("params".to_string())) {
      if let Some(Yaml::String(layout)) = params.get_mut(&Yaml::String("layout".to_string())) {
        let converted = replace_legacy_form_syntax_with_new_one(layout);
        if &converted != layout {
          layout.clear();
          layout.push_str(&converted);
        }
      }
    }
  }
}

lazy_static! {
  static ref LEGACY_FIELD_REGEX: Regex = Regex::new(r"\{\{(?P<name>.*?)\}\}").unwrap();
}

fn replace_legacy_form_syntax_with_new_one(layout: &str) -> String {
  LEGACY_FIELD_REGEX
    .replace_all(layout, |caps: &Captures| {
      let field_name = caps.name("name").unwrap().as_str();
      format!("[[{field_name}]]")
    })
    .to_string()
}

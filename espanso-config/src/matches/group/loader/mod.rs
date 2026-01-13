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
use std::{path::Path, sync::LazyLock};
use thiserror::Error;

use crate::error::NonFatalErrorSet;

use self::yaml::YAMLImporter;

use super::MatchGroup;

pub mod yaml;

trait Importer {
    fn is_supported(&self, extension: &str) -> bool;
    fn load_group(
        &self,
        path: &Path,
        config: &dyn crate::config::Config,
    ) -> Result<(MatchGroup, Option<NonFatalErrorSet>)>;
}

static IMPORTERS: LazyLock<Vec<Box<dyn Importer + Sync + Send>>> =
    LazyLock::new(|| vec![Box::new(YAMLImporter::new())]);

pub fn load_match_group(
    path: &Path,
    config: &dyn crate::config::Config,
) -> Result<(MatchGroup, Option<NonFatalErrorSet>)> {
    if let Some(extension) = path.extension() {
        let extension = extension.to_string_lossy().to_lowercase();

        let importer = IMPORTERS
            .iter()
            .find(|importer| importer.is_supported(&extension));

        match importer {
            Some(importer) => match importer.load_group(path, config) {
                Ok((group, non_fatal_error_set)) => Ok((group, non_fatal_error_set)),
                Err(err) => Err(LoadError::ParsingError(err).into()),
            },
            None => Err(LoadError::InvalidFormat.into()),
        }
    } else {
        Err(LoadError::MissingExtension.into())
    }
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("missing extension in match group file")]
    MissingExtension,

    #[error("invalid match group format")]
    InvalidFormat,

    #[error(transparent)]
    ParsingError(anyhow::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::tests::use_test_directory;

    // MockConfig for testing
    struct MockConfig;

    impl crate::config::Config for MockConfig {
        fn id(&self) -> i32 {
            0
        }
        fn label(&self) -> &str {
            "mock"
        }
        fn match_paths(&self) -> &[String] {
            &[]
        }
        fn backend(&self) -> crate::config::Backend {
            crate::config::Backend::Inject
        }
        fn enable(&self) -> bool {
            true
        }
        fn clipboard_threshold(&self) -> usize {
            100
        }
        fn pre_paste_delay(&self) -> usize {
            100
        }
        fn paste_shortcut_event_delay(&self) -> usize {
            10
        }
        fn paste_shortcut(&self) -> Option<String> {
            None
        }
        fn disable_x11_fast_inject(&self) -> bool {
            false
        }
        fn toggle_key(&self) -> Option<crate::config::ToggleKey> {
            None
        }
        fn auto_restart(&self) -> bool {
            true
        }
        fn preserve_clipboard(&self) -> bool {
            true
        }
        fn restore_clipboard_delay(&self) -> usize {
            300
        }
        fn inject_delay(&self) -> Option<usize> {
            None
        }
        fn key_delay(&self) -> Option<usize> {
            None
        }
        fn evdev_modifier_delay(&self) -> Option<usize> {
            None
        }
        fn word_separators(&self) -> Vec<String> {
            vec![" ".to_string()]
        }
        fn backspace_limit(&self) -> usize {
            5
        }
        fn apply_patch(&self) -> bool {
            true
        }
        fn keyboard_layout(&self) -> Option<crate::config::RMLVOConfig> {
            None
        }
        fn search_trigger(&self) -> Option<String> {
            None
        }
        fn search_shortcut(&self) -> Option<String> {
            None
        }
        fn undo_backspace(&self) -> bool {
            true
        }
        fn show_notifications(&self) -> bool {
            true
        }
        fn show_icon(&self) -> bool {
            true
        }
        fn secure_input_notification(&self) -> bool {
            true
        }
        fn stats_enabled(&self) -> bool {
            true
        }
        fn post_form_delay(&self) -> usize {
            200
        }
        fn max_form_width(&self) -> usize {
            800
        }
        fn max_form_height(&self) -> usize {
            600
        }
        fn max_regex_buffer_size(&self) -> usize {
            30
        }
        fn post_search_delay(&self) -> usize {
            200
        }
        fn emulate_alt_codes(&self) -> bool {
            false
        }
        fn x11_use_xclip_backend(&self) -> bool {
            false
        }
        fn x11_use_xdotool_backend(&self) -> bool {
            false
        }
        fn win32_exclude_orphan_events(&self) -> bool {
            true
        }
        fn win32_keyboard_layout_cache_interval(&self) -> i64 {
            2000
        }
        fn is_match(&self, _app: &crate::config::AppProperties) -> bool {
            true
        }

        fn triggermarker_prefix(&self) -> Option<String> {
            None
        }
        fn triggermarker_suffix(&self) -> Option<String> {
            None
        }
        fn triggermarker_replace_mode(&self) -> String {
            "agnostic".to_string()
        }
        fn triggermarker_prefix_replace_mode(&self) -> Option<String> {
            None
        }
        fn triggermarker_suffix_replace_mode(&self) -> Option<String> {
            None
        }
        fn triggermarker_smart_chars(&self) -> Vec<String> {
            vec![":".to_string(), ";".to_string()]
        }
        fn triggermarker_smart_remove_multiple(&self) -> bool {
            false
        }
    }

    #[test]
    fn load_group_invalid_format() {
        use_test_directory(|_, match_dir, _| {
            let file = match_dir.join("base.invalid");
            std::fs::write(&file, "test").unwrap();

            assert!(matches!(
                load_match_group(&file, &MockConfig)
                    .unwrap_err()
                    .downcast::<LoadError>()
                    .unwrap(),
                LoadError::InvalidFormat
            ));
        });
    }

    #[test]
    fn load_group_missing_extension() {
        use_test_directory(|_, match_dir, _| {
            let file = match_dir.join("base");
            std::fs::write(&file, "test").unwrap();

            assert!(matches!(
                load_match_group(&file, &MockConfig)
                    .unwrap_err()
                    .downcast::<LoadError>()
                    .unwrap(),
                LoadError::MissingExtension
            ));
        });
    }

    #[test]
    fn load_group_parsing_error() {
        use_test_directory(|_, match_dir, _| {
            let file = match_dir.join("base.yml");
            std::fs::write(&file, "test").unwrap();

            assert!(matches!(
                load_match_group(&file, &MockConfig)
                    .unwrap_err()
                    .downcast::<LoadError>()
                    .unwrap(),
                LoadError::ParsingError(_)
            ));
        });
    }

    #[test]
    fn load_group_yaml_format() {
        use_test_directory(|_, match_dir, _| {
            let file = match_dir.join("base.yml");
            std::fs::write(
                &file,
                r#"
      matches:
        - trigger: "hello"
          replace: "world"
      "#,
            )
            .unwrap();

            assert_eq!(
                load_match_group(&file, &MockConfig)
                    .unwrap()
                    .0
                    .matches
                    .len(),
                1
            );
        });
    }

    #[test]
    fn load_group_yaml_format_2() {
        use_test_directory(|_, match_dir, _| {
            let file = match_dir.join("base.yaml");
            std::fs::write(
                &file,
                r#"
      matches:
        - trigger: "hello"
          replace: "world"
      "#,
            )
            .unwrap();

            assert_eq!(
                load_match_group(&file, &MockConfig)
                    .unwrap()
                    .0
                    .matches
                    .len(),
                1
            );
        });
    }

    #[test]
    fn load_group_yaml_format_casing() {
        use_test_directory(|_, match_dir, _| {
            let file = match_dir.join("base.YML");
            std::fs::write(
                &file,
                r#"
      matches:
        - trigger: "hello"
          replace: "world"
      "#,
            )
            .unwrap();

            assert_eq!(
                load_match_group(&file, &MockConfig)
                    .unwrap()
                    .0
                    .matches
                    .len(),
                1
            );
        });
    }
}

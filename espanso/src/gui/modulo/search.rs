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

use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, convert::TryInto};

use crate::gui::{SearchItem, SearchUI};

use super::manager::ModuloManager;

pub trait ModuloSearchUIOptionProvider {
    fn get_post_search_delay(&self) -> usize;
}

pub struct ModuloSearchUI<'a> {
    manager: &'a ModuloManager,
    option_provider: &'a dyn ModuloSearchUIOptionProvider,
}

impl<'a> ModuloSearchUI<'a> {
    pub fn new(
        manager: &'a ModuloManager,
        option_provider: &'a dyn ModuloSearchUIOptionProvider,
    ) -> Self {
        Self {
            manager,
            option_provider,
        }
    }
}

impl SearchUI for ModuloSearchUI<'_> {
    fn show(&self, items: &[SearchItem], hint: Option<&str>) -> anyhow::Result<Option<String>> {
        let modulo_config = ModuloSearchConfig {
            title: "espanso",
            hint,
            items: convert_items(items),
        };

        let json_config = serde_json::to_string(&modulo_config)?;
        // The helper process can leave another app (such as Finder) frontmost
        // when it exits. Remember the target before opening SearchUI.
        #[cfg(target_os = "macos")]
        let frontmost_pid = espanso_info::get_frontmost_application_pid();

        let output = self
            .manager
            .invoke(&["search", "-j", "-i", "-"], &json_config)?;

        process_search_output(
            &output,
            || {
                #[cfg(target_os = "macos")]
                if let Some(pid) = frontmost_pid {
                    if !espanso_info::activate_application(pid) {
                        log::warn!("unable to restore application focus after SearchUI selection");
                    }
                }
            },
            || {
                let post_search_delay = self.option_provider.get_post_search_delay();
                if post_search_delay > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(
                        post_search_delay.try_into().unwrap(),
                    ));
                }
            },
        )
    }
}

// Keep selection handling and its ordering testable without opening a GUI or
// changing the frontmost application in the test runner.
fn process_search_output(
    output: &str,
    restore_focus: impl FnOnce(),
    post_search_delay: impl FnOnce(),
) -> anyhow::Result<Option<String>> {
    let json: Result<HashMap<String, Value>, _> = serde_json::from_str(output);
    let result = match json {
        Ok(json) => {
            if let Some(Value::String(selected_id)) = json.get("selected") {
                Ok(Some(selected_id.clone()))
            } else {
                Ok(None)
            }
        }
        Err(error) => Err(error.into()),
    };

    // Escape and losing focus produce no selection. In particular, do not
    // reactivate the original app when the user deliberately switched away.
    if matches!(&result, Ok(Some(_))) {
        restore_focus();
    }
    post_search_delay();

    result
}

#[derive(Debug, Serialize)]
struct ModuloSearchConfig<'a> {
    title: &'a str,
    hint: Option<&'a str>,
    items: Vec<ModuloSearchItemConfig<'a>>,
}

#[derive(Debug, Serialize)]
struct ModuloSearchItemConfig<'a> {
    id: &'a str,
    label: &'a str,
    trigger: Option<&'a str>,
    search_terms: Vec<&'a str>,
    is_builtin: bool,
}

fn convert_items(items: &'_ [SearchItem]) -> Vec<ModuloSearchItemConfig<'_>> {
    items
        .iter()
        .map(|item| ModuloSearchItemConfig {
            id: &item.id,
            label: &item.label,
            trigger: item.tag.as_deref(),
            search_terms: if item.additional_search_terms.is_empty() {
                vec![]
            } else {
                item.additional_search_terms
                    .iter()
                    .map(String::as_str)
                    .collect()
            },
            is_builtin: item.is_builtin,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::process_search_output;
    use std::cell::RefCell;

    #[test]
    fn selection_restores_focus_before_delay_and_returning_to_expand() {
        let events = RefCell::new(Vec::new());
        let result = process_search_output(
            r#"{"selected":"match-id"}"#,
            || events.borrow_mut().push("restore"),
            || events.borrow_mut().push("delay"),
        )
        .unwrap();
        events.borrow_mut().push("return");

        assert_eq!(result.as_deref(), Some("match-id"));
        assert_eq!(*events.borrow(), ["restore", "delay", "return"]);
    }

    #[test]
    fn cancellation_does_not_restore_focus_and_preserves_delay() {
        // SearchUI returns selected: null for both Escape and loss of focus.
        let events = RefCell::new(Vec::new());
        let result = process_search_output(
            r#"{"selected":null}"#,
            || panic!("must not restore focus after cancellation"),
            || events.borrow_mut().push("delay"),
        )
        .unwrap();

        assert!(result.is_none());
        assert_eq!(*events.borrow(), ["delay"]);
    }

    #[test]
    fn missing_or_non_string_selection_does_not_restore_focus() {
        for output in ["{}", r#"{"selected":42}"#, r#"{"selected":false}"#] {
            let mut delayed = false;
            let result = process_search_output(
                output,
                || panic!("must not restore focus without a selection"),
                || delayed = true,
            )
            .unwrap();

            assert!(result.is_none());
            assert!(delayed);
        }
    }

    #[test]
    fn invalid_output_preserves_error_and_delay_without_restoring_focus() {
        for output in ["not json", "", "[]", "null"] {
            let mut delayed = false;
            let result = process_search_output(
                output,
                || panic!("must not restore focus after a parse error"),
                || delayed = true,
            );

            assert!(result.is_err());
            assert!(delayed);
        }
    }
}

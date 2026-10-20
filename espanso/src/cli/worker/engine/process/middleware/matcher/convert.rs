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

use espanso_config::{
    config::ConfigStore,
    matches::{
        store::{MatchSet, MatchStore},
        MatchCause,
    },
};
use espanso_detect::hotkey::HotKey;
use espanso_match::{
    regex::RegexMatch,
    rolling::{RollingMatch, StringMatchOptions},
};
use log::error;

use crate::cli::worker::builtin::BuiltInMatch;

pub struct MatchConverter<'a> {
    config_store: &'a dyn ConfigStore,
    match_store: &'a dyn MatchStore,
    builtin_matches: &'a [BuiltInMatch],
}

impl<'a> MatchConverter<'a> {
    pub fn new(
        config_store: &'a dyn ConfigStore,
        match_store: &'a dyn MatchStore,
        builtin_matches: &'a [BuiltInMatch],
    ) -> Self {
        Self {
            config_store,
            match_store,
            builtin_matches,
        }
    }

    // TODO: (might need to move the conversion logic into a separate function)
    pub fn get_rolling_matches(&self) -> Vec<RollingMatch<i32>> {
        let match_set = self.global_match_set();
        let mut matches = Vec::new();

        // First convert configuration (user-defined) matches
        for m in match_set.matches {
            if let MatchCause::Trigger(cause) = &m.cause {
                for trigger in &cause.triggers {
                    matches.push(RollingMatch::from_string(
                        m.id,
                        trigger,
                        &StringMatchOptions {
                            case_insensitive: cause
                                .case_sensitive
                                .map_or(cause.propagate_case, |sensitive| !sensitive),
                            left_word: cause.left_word,
                            right_word: cause.right_word,
                        },
                    ));
                }
            }
        }

        // Then convert built-in ones
        for m in self.builtin_matches {
            for trigger in &m.triggers {
                matches.push(RollingMatch::from_string(
                    m.id,
                    trigger,
                    &StringMatchOptions::default(),
                ));
            }
        }

        matches
    }

    // TODO: (might need to move the conversion logic into a separate function)
    pub fn get_regex_matches(&self) -> Vec<RegexMatch<i32>> {
        let match_set = self.global_match_set();
        let mut matches = Vec::new();

        for m in match_set.matches {
            if let MatchCause::Regex(cause) = &m.cause {
                matches.push(RegexMatch::new(m.id, &cause.regex));
            }
        }

        matches
    }

    pub fn get_hotkeys(&self) -> Vec<HotKey> {
        let mut hotkeys = Vec::new();

        // TODO: read user-defined matches

        // Then convert built-in ones
        for m in self.builtin_matches {
            if let Some(hotkey) = &m.hotkey {
                match HotKey::new(m.id, hotkey) {
                    Ok(hotkey) => hotkeys.push(hotkey),
                    Err(err) => {
                        error!("unable to register hotkey: {hotkey}, with error: {err}");
                    }
                }
            }
        }

        hotkeys
    }

    fn global_match_set(&'_ self) -> MatchSet<'_> {
        let paths = self.config_store.get_all_match_paths();
        self.match_store
            .query(&paths.into_iter().collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, fs, sync::Arc};

    use espanso_config::{config::Config, matches::Match};
    use espanso_engine::{
        event::input::Key,
        process::{Matcher, MatcherEvent, Renderer},
    };

    use crate::cli::worker::engine::process::middleware::{
        matcher::rolling::{RollingMatcherAdapter, RollingMatcherAdapterOptions},
        render::{ConfigProvider, MatchProvider, RendererAdapter},
    };

    struct TestProviders<'a> {
        config_store: &'a dyn ConfigStore,
        match_store: &'a dyn MatchStore,
    }

    impl<'a> MatchProvider<'a> for TestProviders<'a> {
        fn matches(&self) -> Vec<&'a Match> {
            let paths = self.config_store.get_all_match_paths();
            self.match_store
                .query(&paths.into_iter().collect::<Vec<_>>())
                .matches
        }

        fn get(&self, id: i32) -> Option<&'a Match> {
            self.matches().into_iter().find(|m| m.id == id)
        }
    }

    impl ConfigProvider<'_> for TestProviders<'_> {
        fn configs(&self) -> Vec<(Arc<dyn Config>, MatchSet<'_>)> {
            self.config_store
                .configs()
                .into_iter()
                .map(|config| {
                    let matches = self.match_store.query(config.match_paths());
                    (config, matches)
                })
                .collect()
        }

        fn active(&self) -> (Arc<dyn Config>, MatchSet<'_>) {
            let config = self.config_store.default();
            let matches = self.match_store.query(config.match_paths());
            (config, matches)
        }
    }

    // Exercise YAML loading, production matcher conversion, typed events and rendering together.
    fn expand(yaml: &str, config: &str, input: &str) -> Vec<(String, String)> {
        let directory = tempdir::TempDir::new("espanso-trigger-case").unwrap();
        fs::create_dir(directory.path().join("config")).unwrap();
        fs::create_dir(directory.path().join("match")).unwrap();
        fs::write(directory.path().join("config/default.yml"), config).unwrap();
        fs::write(directory.path().join("match/base.yml"), yaml).unwrap();
        let (config_store, match_store, errors) = espanso_config::load(directory.path()).unwrap();
        assert!(errors.is_empty(), "configuration errors: {errors:?}");

        let converter = MatchConverter::new(config_store.as_ref(), match_store.as_ref(), &[]);
        let matcher = RollingMatcherAdapter::new(
            &converter.get_rolling_matches(),
            RollingMatcherAdapterOptions {
                char_word_separators: config_store.default().word_separators(),
            },
        );
        let providers = TestProviders {
            config_store: config_store.as_ref(),
            match_store: match_store.as_ref(),
        };
        let renderer = espanso_render::create(Vec::new());
        let renderer = RendererAdapter::new(&providers, &providers, &renderer);
        let mut state = None;
        let mut expansions = Vec::new();

        for character in input.chars() {
            let (next_state, results) = matcher.process(
                state.as_ref(),
                &MatcherEvent::Key {
                    key: Key::Other(0),
                    chars: Some(character.to_string()),
                },
            );
            state = Some(next_state);
            for result in results {
                let output = renderer
                    .render(result.id, Some(&result.trigger), HashMap::new())
                    .unwrap();
                expansions.push((result.trigger, output));
            }
        }
        expansions
    }

    #[test]
    fn case_sensitivity_and_output_propagation_are_independent() {
        for case_sensitive in [None, Some(false), Some(true)] {
            for propagate_case in [false, true] {
                let setting = case_sensitive
                    .map(|value| format!("    case_sensitive: {value}\n"))
                    .unwrap_or_default();
                let yaml = format!(
                    "matches:\n  - trigger: brb/\n    replace: be right back\n\
                         {setting}    propagate_case: {propagate_case}\n"
                );
                for (input, propagated) in [
                    ("brb/", "be right back"),
                    ("Brb/", "Be right back"),
                    ("BRB/", "BE RIGHT BACK"),
                    ("bRb/", "be right back"),
                ] {
                    let results = expand(&yaml, "", input);
                    let should_match = input == "brb/"
                        || case_sensitive == Some(false)
                        || (case_sensitive.is_none() && propagate_case);
                    if should_match {
                        let expected = if propagate_case {
                            propagated
                        } else {
                            "be right back"
                        };
                        assert_eq!(results, vec![(input.to_string(), expected.to_string())]);
                    } else {
                        assert!(results.is_empty(), "unexpected expansion of {input}");
                    }
                }
            }
        }
    }

    #[test]
    fn explicit_sensitive_matching_keeps_output_propagation() {
        let yaml = "matches:\n  - trigger: Brb/\n    replace: be right back\n    case_sensitive: true\n    propagate_case: true\n";
        assert!(expand(yaml, "", "brb/").is_empty());
        assert!(expand(yaml, "", "BRB/").is_empty());
        assert_eq!(
            expand(yaml, "", "Brb/"),
            vec![("Brb/".to_string(), "Be right back".to_string())]
        );
    }

    #[test]
    fn multiple_triggers_preserve_fixed_output_for_every_case() {
        let yaml = "matches:\n  - triggers: [brb/, back/]\n    replace: I'll be right back\n    case_sensitive: false\n";
        for input in [
            "brb/", "brB/", "bRb/", "bRB/", "Brb/", "BrB/", "BRb/", "BRB/", "back/", "BaCk/",
            "BACK/",
        ] {
            assert_eq!(
                expand(yaml, "", input),
                vec![(input.to_string(), "I'll be right back".to_string())]
            );
        }
    }

    #[test]
    fn insensitive_word_match_waits_for_separator() {
        let yaml = "matches:\n  - trigger: wordpress\n    replace: WordPress\n    case_sensitive: false\n    word: true\n";
        for input in ["WoRdPrEsS", "xWoRdPrEsS ", "WoRdPrEsSx "] {
            assert!(expand(yaml, "", input).is_empty());
        }
        assert_eq!(
            expand(yaml, "", "WoRdPrEsS "),
            vec![("WoRdPrEsS ".to_string(), "WordPress".to_string())]
        );
        assert_eq!(
            expand(yaml, "word_separators: ['#']", "WORDPRESS#"),
            vec![("WORDPRESS#".to_string(), "WordPress".to_string())]
        );
        assert!(expand(yaml, "word_separators: ['#']", "WORDPRESS ").is_empty());
    }

    #[test]
    fn insensitive_match_preserves_one_sided_boundaries() {
        let left = "matches:\n  - trigger: brb/\n    replace: back\n    case_sensitive: false\n    left_word: true\n";
        assert_eq!(expand(left, "", "BRB/").len(), 1);
        assert!(expand(left, "", "xBRB/").is_empty());

        let right = "matches:\n  - trigger: wordpress\n    replace: WordPress\n    case_sensitive: false\n    right_word: true\n";
        assert!(expand(right, "", "WORDPRESS").is_empty());
        assert_eq!(expand(right, "", "xWORDPRESS ").len(), 1);
    }

    #[test]
    fn insensitive_match_handles_non_ascii_and_preserves_typed_trigger() {
        let yaml = "matches:\n  - trigger: äpfel/\n    replace: Äpfel\n    case_sensitive: false\n";
        assert_eq!(
            expand(yaml, "", "ÄPFEL/"),
            vec![("ÄPFEL/".to_string(), "Äpfel".to_string())]
        );
    }
}

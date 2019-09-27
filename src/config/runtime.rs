/*
 * This file is part of espanso.
 *
 * Copyright (C) 2019 Federico Terzi
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

use regex::Regex;
use crate::system::SystemManager;
use std::cell::RefCell;
use std::time::SystemTime;
use log::{debug, warn};
use super::{Configs, ConfigSet};
use crate::matcher::Match;

pub struct RuntimeConfigManager<'a, S: SystemManager> {
    set: ConfigSet,

    // Filter regexps
    title_regexps: Vec<Option<Regex>>,
    class_regexps: Vec<Option<Regex>>,
    exec_regexps: Vec<Option<Regex>>,

    system_manager: S,

    // Cache
    last_config_update: RefCell<SystemTime>,
    last_config: RefCell<Option<&'a Configs>>
}

impl <'a, S: SystemManager> RuntimeConfigManager<'a, S> {
    pub fn new<'b>(set: ConfigSet, system_manager: S) -> RuntimeConfigManager<'b, S> {
        // Compile all the regexps
        let title_regexps = set.specific.iter().map(
            |config| {
                if config.filter_title.is_empty() {
                    None
                }else{
                    let res = Regex::new(&config.filter_title);
                    if let Ok(regex) = res {
                        Some(regex)
                    }else{
                        warn!("Invalid regex in 'filter_title' field of configuration {}, ignoring it...", config.name);
                        None
                    }
                }
            }
        ).collect();

        let class_regexps = set.specific.iter().map(
            |config| {
                if config.filter_class.is_empty() {
                    None
                }else{
                    let res = Regex::new(&config.filter_class);
                    if let Ok(regex) = res {
                        Some(regex)
                    }else{
                        warn!("Invalid regex in 'filter_class' field of configuration {}, ignoring it...", config.name);
                        None
                    }
                }
            }
        ).collect();

        let exec_regexps = set.specific.iter().map(
            |config| {
                if config.filter_exec.is_empty() {
                    None
                }else{
                    let res = Regex::new(&config.filter_exec);
                    if let Ok(regex) = res {
                        Some(regex)
                    }else{
                        warn!("Invalid regex in 'filter_exec' field of configuration {}, ignoring it...", config.name);
                        None
                    }
                }
            }
        ).collect();

        let last_config_update = RefCell::new(SystemTime::now());
        let last_config = RefCell::new(None);

        RuntimeConfigManager {
            set,
            title_regexps,
            class_regexps,
            exec_regexps,
            system_manager,
            last_config_update,
            last_config
        }
    }

    fn calculate_active_config(&'a self) -> &'a Configs {
        // TODO: optimize performance by avoiding some of these checks if no Configs use the filters

        debug!("Requested config for window:");

        let active_title = self.system_manager.get_current_window_title();

        if let Some(title) = active_title {
            debug!("=> Title: '{}'", title);

            for (i, regex) in self.title_regexps.iter().enumerate() {
                if let Some(regex) = regex {
                    if regex.is_match(&title) {
                        debug!("Matched 'filter_title' for '{}' config, using custom settings.",
                               self.set.specific[i].name);

                        return &self.set.specific[i]
                    }
                }
            }
        }

        let active_executable = self.system_manager.get_current_window_executable();

        if let Some(executable) = active_executable {
            debug!("=> Executable: '{}'", executable);

            for (i, regex) in self.exec_regexps.iter().enumerate() {
                if let Some(regex) = regex {
                    if regex.is_match(&executable) {
                        debug!("Matched 'filter_exec' for '{}' config, using custom settings.",
                               self.set.specific[i].name);

                        return &self.set.specific[i]
                    }
                }
            }
        }

        let active_class = self.system_manager.get_current_window_class();

        if let Some(class) = active_class {
            debug!("=> Class: '{}'", class);

            for (i, regex) in self.class_regexps.iter().enumerate() {
                if let Some(regex) = regex {
                    if regex.is_match(&class) {
                        debug!("Matched 'filter_class' for '{}' config, using custom settings.",
                               self.set.specific[i].name);

                        return &self.set.specific[i]
                    }
                }
            }
        }

        // No matches, return the default mapping
        debug!("No matches for custom configs, using default settings.");
        &self.set.default
    }
}

impl <'a, S: SystemManager> super::ConfigManager<'a> for RuntimeConfigManager<'a, S> {
    fn active_config(&'a self) -> &'a Configs {
        let mut last_config_update = self.last_config_update.borrow_mut();
        if let Ok(elapsed) = (*last_config_update).elapsed() {
            *last_config_update = SystemTime::now();

            if elapsed.as_millis() < self.set.default.config_caching_interval as u128 {
                let last_config = self.last_config.borrow();
                if let Some(cached_config) = *last_config {
                    debug!("Using cached config");
                    return cached_config;
                }
            }
        }

        let config = self.calculate_active_config();

        let mut last_config = self.last_config.borrow_mut();
        *last_config = Some(config);

        config
    }

    fn default_config(&'a self) -> &'a Configs {
        &self.set.default
    }

    fn matches(&'a self) -> &'a Vec<Match> {
        &self.active_config().matches
    }
}

// TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};
    use crate::config::{DEFAULT_CONFIG_FILE_NAME, DEFAULT_CONFIG_FILE_CONTENT};
    use std::fs;
    use std::path::PathBuf;
    use crate::config::ConfigManager;
    use crate::config::tests::{create_temp_espanso_directory, create_temp_file_in_dir, create_user_config_file};

    struct DummySystemManager {
        title: RefCell<String>,
        class: RefCell<String>,
        exec: RefCell<String>,
    }
    impl SystemManager for DummySystemManager {
        fn get_current_window_title(&self) -> Option<String> {
            Some(self.title.borrow().clone())
        }
        fn get_current_window_class(&self) -> Option<String> {
            Some(self.class.borrow().clone())
        }
        fn get_current_window_executable(&self) -> Option<String> {
            Some(self.exec.borrow().clone())
        }
    }
    impl DummySystemManager {
        pub fn new_custom(title: &str, class: &str, exec: &str) -> DummySystemManager {
            DummySystemManager{
                title: RefCell::new(title.to_owned()),
                class: RefCell::new(class.to_owned()),
                exec: RefCell::new(exec.to_owned())
            }
        }

        pub fn new() -> DummySystemManager {
            DummySystemManager::new_custom("title", "class", "exec")
        }

        pub fn change(&self, title: &str, class: &str, exec: &str) {
            *self.title.borrow_mut() = title.to_owned();
            *self.class.borrow_mut() = class.to_owned();
            *self.exec.borrow_mut() = exec.to_owned();
        }
    }

    #[test]
    fn test_runtime_constructor_regex_load_correctly() {
        let tmp_dir = create_temp_espanso_directory();

        let specific_path = create_user_config_file(&tmp_dir.path(), "specific.yml", r###"
        name: myname1
        filter_exec: "Title"
        "###);

        let specific_path2 = create_user_config_file(&tmp_dir.path(), "specific2.yml", r###"
        name: myname2
        filter_title: "Yeah"
        filter_class: "Car"
        "###);

        let specific_path3 = create_user_config_file(&tmp_dir.path(), "specific3.yml", r###"
        name: myname3
        filter_title: "Nice"
        "###);

        let config_set = ConfigSet::load(tmp_dir.path());
        assert!(config_set.is_ok());

        let dummy_system_manager = DummySystemManager::new();

        let config_manager = RuntimeConfigManager::new(config_set.unwrap(), dummy_system_manager);

        let sp1index = config_manager.set.specific
            .iter().position(|x| x.name == "myname1").unwrap();
        let sp2index = config_manager.set.specific
            .iter().position(|x| x.name == "myname2").unwrap();
        let sp3index = config_manager.set.specific
            .iter().position(|x| x.name == "myname3").unwrap();

        assert_eq!(config_manager.exec_regexps.len(), 3);
        assert_eq!(config_manager.title_regexps.len(), 3);
        assert_eq!(config_manager.class_regexps.len(), 3);

        assert!(config_manager.class_regexps[sp1index].is_none());
        assert!(config_manager.class_regexps[sp2index].is_some());
        assert!(config_manager.class_regexps[sp3index].is_none());

        assert!(config_manager.title_regexps[sp1index].is_none());
        assert!(config_manager.title_regexps[sp2index].is_some());
        assert!(config_manager.title_regexps[sp3index].is_some());

        assert!(config_manager.exec_regexps[sp1index].is_some());
        assert!(config_manager.exec_regexps[sp2index].is_none());
        assert!(config_manager.exec_regexps[sp3index].is_none());
    }

    #[test]
    fn test_runtime_constructor_malformed_regexes_are_ignored() {
        let tmp_dir = create_temp_espanso_directory();

        let specific_path = create_user_config_file(&tmp_dir.path(), "specific.yml", r###"
        name: myname1
        filter_exec: "[`-_]"
        "###);

        let specific_path2 = create_user_config_file(&tmp_dir.path(), "specific2.yml", r###"
        name: myname2
        filter_title: "[`-_]"
        filter_class: "Car"
        "###);

        let specific_path3 = create_user_config_file(&tmp_dir.path(), "specific3.yml", r###"
        name: myname3
        filter_title: "Nice"
        "###);

        let config_set = ConfigSet::load(tmp_dir.path());
        assert!(config_set.is_ok());

        let dummy_system_manager = DummySystemManager::new();

        let config_manager = RuntimeConfigManager::new(config_set.unwrap(), dummy_system_manager);

        let sp1index = config_manager.set.specific
            .iter().position(|x| x.name == "myname1").unwrap();
        let sp2index = config_manager.set.specific
            .iter().position(|x| x.name == "myname2").unwrap();
        let sp3index = config_manager.set.specific
            .iter().position(|x| x.name == "myname3").unwrap();

        assert_eq!(config_manager.exec_regexps.len(), 3);
        assert_eq!(config_manager.title_regexps.len(), 3);
        assert_eq!(config_manager.class_regexps.len(), 3);

        assert!(config_manager.class_regexps[sp1index].is_none());
        assert!(config_manager.class_regexps[sp2index].is_some());
        assert!(config_manager.class_regexps[sp3index].is_none());

        assert!(config_manager.title_regexps[sp1index].is_none());
        assert!(config_manager.title_regexps[sp2index].is_none());
        assert!(config_manager.title_regexps[sp3index].is_some());

        assert!(config_manager.exec_regexps[sp1index].is_none());
        assert!(config_manager.exec_regexps[sp2index].is_none());
        assert!(config_manager.exec_regexps[sp3index].is_none());
    }

    #[test]
    fn test_runtime_calculate_active_config_specific_title_match() {
        let tmp_dir = create_temp_espanso_directory();

        let specific_path = create_user_config_file(&tmp_dir.path(), "specific.yml", r###"
        name: chrome
        filter_title: "Chrome"
        "###);

        let config_set = ConfigSet::load(tmp_dir.path());
        assert!(config_set.is_ok());

        let dummy_system_manager = DummySystemManager::new_custom("Google Chrome", "Chrome", "C:\\Path\\chrome.exe");

        let config_manager = RuntimeConfigManager::new(config_set.unwrap(), dummy_system_manager);

        assert_eq!(config_manager.calculate_active_config().name, "chrome");
    }

    fn test_runtime_calculate_active_config_specific_class_match() {
        let tmp_dir = create_temp_espanso_directory();

        let specific_path = create_user_config_file(&tmp_dir.path(), "specific.yml", r###"
        name: chrome
        filter_class: "Chrome"
        "###);

        let config_set = ConfigSet::load(tmp_dir.path());
        assert!(config_set.is_ok());

        let dummy_system_manager = DummySystemManager::new_custom("Google Chrome", "Chrome", "C:\\Path\\chrome.exe");

        let config_manager = RuntimeConfigManager::new(config_set.unwrap(), dummy_system_manager);

        assert_eq!(config_manager.calculate_active_config().name, "chrome");
    }

    fn test_runtime_calculate_active_config_specific_exec_match() {
        let tmp_dir = create_temp_espanso_directory();

        let specific_path = create_user_config_file(&tmp_dir.path(), "specific.yml", r###"
        name: chrome
        filter_exec: "chrome.exe"
        "###);

        let config_set = ConfigSet::load(tmp_dir.path());
        assert!(config_set.is_ok());

        let dummy_system_manager = DummySystemManager::new_custom("Google Chrome", "Chrome", "C:\\Path\\chrome.exe");

        let config_manager = RuntimeConfigManager::new(config_set.unwrap(), dummy_system_manager);

        assert_eq!(config_manager.calculate_active_config().name, "chrome");
    }

    fn test_runtime_calculate_active_config_specific_multi_filter_match() {
        let tmp_dir = create_temp_espanso_directory();

        let specific_path = create_user_config_file(&tmp_dir.path(), "specific.yml", r###"
        name: chrome
        filter_class: Browser
        filter_exec: "firefox.exe"
        "###);

        let config_set = ConfigSet::load(tmp_dir.path());
        assert!(config_set.is_ok());

        let dummy_system_manager = DummySystemManager::new_custom("Google Chrome", "Browser", "C:\\Path\\chrome.exe");

        let config_manager = RuntimeConfigManager::new(config_set.unwrap(), dummy_system_manager);

        assert_eq!(config_manager.calculate_active_config().name, "chrome");
    }

    #[test]
    fn test_runtime_calculate_active_config_no_match() {
        let tmp_dir = create_temp_espanso_directory();

        let specific_path = create_user_config_file(&tmp_dir.path(), "specific.yml", r###"
        name: firefox
        filter_title: "Firefox"
        "###);

        let config_set = ConfigSet::load(tmp_dir.path());
        assert!(config_set.is_ok());

        let dummy_system_manager = DummySystemManager::new_custom("Google Chrome", "Chrome", "C:\\Path\\chrome.exe");

        let config_manager = RuntimeConfigManager::new(config_set.unwrap(), dummy_system_manager);

        assert_eq!(config_manager.calculate_active_config().name, "default");
    }

    #[test]
    fn test_runtime_active_config_cache() {
        let tmp_dir = create_temp_espanso_directory();

        let specific_path = create_user_config_file(&tmp_dir.path(), "specific.yml", r###"
        name: firefox
        filter_title: "Firefox"
        "###);

        let config_set = ConfigSet::load(tmp_dir.path());
        assert!(config_set.is_ok());

        let dummy_system_manager = DummySystemManager::new_custom("Google Chrome", "Chrome", "C:\\Path\\chrome.exe");

        let config_manager = RuntimeConfigManager::new(config_set.unwrap(), dummy_system_manager);

        assert_eq!(config_manager.active_config().name, "default");
        assert_eq!(config_manager.calculate_active_config().name, "default");

        config_manager.system_manager.change("Firefox", "Browser", "C\\Path\\firefox.exe");

        // Active config should have changed, but not cached one
        assert_eq!(config_manager.calculate_active_config().name, "firefox");
        assert_eq!(config_manager.active_config().name, "default");
    }
}
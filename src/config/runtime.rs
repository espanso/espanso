use regex::Regex;
use crate::system::SystemManager;
use std::cell::RefCell;
use std::time::SystemTime;
use log::{debug, info, warn, error};
use super::{Configs, ConfigSet};
use crate::matcher::Match;

pub struct RuntimeConfigManager<'a, S: SystemManager> {
    set: ConfigSet,
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
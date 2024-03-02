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

use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use espanso_info::{AppInfo, AppInfoProvider};

pub struct CachedAppInfoProvider<'a> {
    app_info_provider: &'a dyn AppInfoProvider,
    caching_interval: Duration,

    cached_info: RefCell<Option<(Instant, AppInfo)>>,
}

impl<'a> CachedAppInfoProvider<'a> {
    pub fn from(app_info_provider: &'a dyn AppInfoProvider, caching_interval: Duration) -> Self {
        Self {
            app_info_provider,
            caching_interval,
            cached_info: RefCell::new(None),
        }
    }
}

impl<'a> AppInfoProvider for CachedAppInfoProvider<'a> {
    fn get_info(&self) -> espanso_info::AppInfo {
        let mut cached_info = self.cached_info.borrow_mut();
        if let Some((instant, cached_value)) = &*cached_info {
            if instant.elapsed() < self.caching_interval {
                // Return cached config
                return cached_value.clone();
            }
        }

        let info = self.app_info_provider.get_info();
        *cached_info = Some((Instant::now(), info.clone()));

        info
    }
}

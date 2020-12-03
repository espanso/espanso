/*
 * This file is part of espanso.
 *
 * Copyright (C) 2020 Federico Terzi
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

use crate::config::Configs;
use log::debug;
use std::sync::atomic::Ordering::Release;
use std::sync::{atomic::AtomicBool, Arc};

pub struct InjectGuard {
    is_injecting: Arc<AtomicBool>,
    post_inject_delay: u64,
}

impl InjectGuard {
    pub fn new(is_injecting: Arc<AtomicBool>, config: &Configs) -> Self {
        debug!("enabling inject guard");

        // Enable the injecting block
        is_injecting.store(true, Release);

        Self {
            is_injecting,
            post_inject_delay: config.post_inject_delay,
        }
    }
}

impl Drop for InjectGuard {
    fn drop(&mut self) {
        // Because the keyinjection is async, we need to wait a bit before
        // giving back the control. Otherwise, the injected actions will be handled back
        // by espanso itself.
        std::thread::sleep(std::time::Duration::from_millis(self.post_inject_delay));

        debug!("releasing inject guard");

        // Re-allow espanso to interpret actions
        self.is_injecting.store(false, Release);
    }
}

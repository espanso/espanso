use crate::config::Configs;
use log::debug;
use std::sync::atomic::Ordering::Release;
use std::sync::{atomic::AtomicBool, Arc};

pub struct InjectGuard {
    is_injecting: Arc<AtomicBool>,
    mac_post_inject_delay: u64,
}

impl InjectGuard {
    pub fn new(is_injecting: Arc<AtomicBool>, config: &Configs) -> Self {
        debug!("enabling inject guard");

        // Enable the injecting block
        is_injecting.store(true, Release);

        Self {
            is_injecting,
            mac_post_inject_delay: config.mac_post_inject_delay,
        }
    }
}

impl Drop for InjectGuard {
    fn drop(&mut self) {
        debug!("releasing inject guard");

        // On macOS, because the keyinjection is async, we need to wait a bit before
        // giving back the control. Otherwise, the injected actions will be handled back
        // by espanso itself.
        if cfg!(target_os = "macos") {
            std::thread::sleep(std::time::Duration::from_millis(self.mac_post_inject_delay));
        }

        // Re-allow espanso to interpret actions
        self.is_injecting.store(false, Release);
    }
}

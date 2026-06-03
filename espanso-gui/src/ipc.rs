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

//! IPC client for communicating with the espanso Worker process.
//!
//! The GUI process connects to the Worker via the existing `espanso-ipc`
//! crate's Unix socket (or Windows named pipe) mechanism.

use std::path::PathBuf;

use anyhow::Result;
use log::{debug, warn};

/// Connection state with the Worker process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerConnectionState {
    Connected,
    Disconnected,
    Connecting,
}

/// Client for communicating with the Worker process via IPC.
pub struct IpcClient {
    runtime_dir: Option<PathBuf>,
    state: WorkerConnectionState,
}

impl IpcClient {
    /// Create a new IPC client.
    pub fn new(runtime_dir: Option<PathBuf>) -> Self {
        IpcClient {
            runtime_dir,
            state: WorkerConnectionState::Disconnected,
        }
    }

    /// Check if we have a connection to the worker.
    pub fn is_connected(&self) -> bool {
        self.state == WorkerConnectionState::Connected
    }

    /// Get the current connection state.
    pub fn state(&self) -> &WorkerConnectionState {
        &self.state
    }

    /// Try to connect to the Worker.
    pub fn try_connect(&mut self) -> Result<bool> {
        if let Some(ref runtime_dir) = self.runtime_dir {
            // The worker IPC server socket is named "espansoworkerv2"
            // We check if the socket/pipe exists
            let socket_path = runtime_dir.join("espansoworkerv2.sock");
            let pipe_path = runtime_dir.join("espansoworkerv2");

            let connected = if cfg!(windows) {
                // Windows: check named pipe
                pipe_path.exists()
            } else {
                // Unix: check socket
                socket_path.exists()
            };

            if connected {
                self.state = WorkerConnectionState::Connected;
                debug!("Connected to Worker IPC");
            } else {
                self.state = WorkerConnectionState::Disconnected;
                debug!("Worker IPC not available");
            }

            Ok(connected)
        } else {
            warn!("No runtime_dir configured, cannot connect to Worker");
            self.state = WorkerConnectionState::Disconnected;
            Ok(false)
        }
    }

    /// Notify the Worker to reload configuration.
    /// This is a fire-and-forget operation.
    pub fn notify_config_changed(&self) -> Result<()> {
        if !self.is_connected() {
            // If Worker is not running, no need to notify — it will
            // pick up changes on next start.
            debug!("Worker not connected, skipping config change notification");
            return Ok(());
        }

        // TODO: Actually send IPC event to Worker
        // For now, this is a stub. The full implementation will use
        // espanso_ipc::create_ipc_client_to_worker() to send a
        // ConfigChanged event.
        debug!("Notified Worker of config change (stub)");
        Ok(())
    }

    /// Send a test expansion request to the Worker.
    pub fn request_test_expansion(
        &self,
        trigger: &str,
        app_title: Option<&str>,
        app_class: Option<&str>,
        app_exec: Option<&str>,
    ) -> Result<Option<String>> {
        if !self.is_connected() {
            return Ok(None);
        }

        debug!("Requesting test expansion for '{}' (stub)", trigger);
        // TODO: Actually send IPC event and wait for response
        let _ = (trigger, app_title, app_class, app_exec);
        Ok(None)
    }

    /// Request stats data from the Worker.
    pub fn request_stats(
        &self,
        _period: &str,
        _top_n: usize,
    ) -> Result<Option<()>> {
        if !self.is_connected() {
            return Ok(None);
        }

        debug!("Requesting stats (stub)");
        // TODO: Actually send IPC event and wait for response
        Ok(None)
    }
}
